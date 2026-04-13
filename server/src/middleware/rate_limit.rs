use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use redis::aio::ConnectionManager;
use std::net::IpAddr;
use std::sync::Arc;
use tracing;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Per-endpoint-group rate limit configuration.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed in the window.
    pub max_requests: u32,
    /// Sliding window size in seconds.
    pub window_secs: u64,
}

// ---------------------------------------------------------------------------
// Client IP extraction
// ---------------------------------------------------------------------------

/// Extract the client IP address, only trusting `X-Forwarded-For` when the
/// connecting peer is in the `trusted_proxies` allowlist.
///
/// When `trusted_proxies` is empty, the header is never consulted — this is
/// the secure default ("don't trust anyone").
pub fn extract_client_ip(req: &Request, trusted_proxies: &[IpAddr]) -> Option<IpAddr> {
    let peer_ip = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip());

    // Only trust X-Forwarded-For from known proxies
    if let Some(peer) = peer_ip {
        if trusted_proxies.contains(&peer) {
            if let Some(forwarded) = req.headers().get("x-forwarded-for") {
                if let Ok(val) = forwarded.to_str() {
                    if let Some(first) = val.split(',').next() {
                        if let Ok(ip) = first.trim().parse::<IpAddr>() {
                            return Some(ip);
                        }
                    }
                }
            }
        }
    }

    peer_ip
}

// ---------------------------------------------------------------------------
// Rate limit key
// ---------------------------------------------------------------------------

/// Build the Redis key used for the sliding window ZSET.
///
/// Format: `rate:{project_id}:{ip}:{endpoint_group}`
pub fn build_rate_limit_key(project_id: &str, ip: &str, endpoint_group: &str) -> String {
    format!("rate:{}:{}:{}", project_id, ip, endpoint_group)
}

// ---------------------------------------------------------------------------
// Middleware
// ---------------------------------------------------------------------------

/// Rate-limiting middleware backed by a Redis sorted-set sliding window.
///
/// This is a *middleware factory* — call it with configuration and a Redis
/// connection to obtain a closure suitable for [`axum::middleware::from_fn`].
///
/// **Algorithm (sliding window using ZSET)**
/// 1. `ZREMRANGEBYSCORE key -inf (now - window)` — remove expired entries.
/// 2. `ZCARD key` — count remaining entries in the window.
/// 3. If count >= max_requests → return **429 Too Many Requests** with
///    a `Retry-After` header.
/// 4. `ZADD key now now` — record the current request.
/// 5. `EXPIRE key window_secs` — ensure the key eventually gets cleaned up.
pub async fn rate_limit_middleware(
    redis: Arc<ConnectionManager>,
    config: RateLimitConfig,
    trusted_proxies: Arc<Vec<IpAddr>>,
    project_id: String,
    endpoint_group: String,
    req: Request,
    next: Next,
) -> Response {
    let ip = extract_client_ip(&req, &trusted_proxies)
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let key = build_rate_limit_key(&project_id, &ip, &endpoint_group);

    match check_rate_limit(&redis, &key, &config).await {
        Ok(true) => {
            // Under the limit — proceed
            next.run(req).await
        }
        Ok(false) => {
            // Exceeded
            let retry_after = config.window_secs.to_string();
            (
                StatusCode::TOO_MANY_REQUESTS,
                [("retry-after", retry_after.as_str())],
                "Rate limit exceeded",
            )
                .into_response()
        }
        Err(e) => {
            // On Redis errors we **allow** the request through (fail-open)
            // to avoid a Redis outage from blocking all traffic.
            tracing::warn!(error = %e, key = %key, "rate limit check failed, allowing request");
            next.run(req).await
        }
    }
}

/// Core sliding-window logic against Redis.
///
/// Returns `Ok(true)` if the request is allowed, `Ok(false)` if rate-limited.
async fn check_rate_limit(
    redis: &ConnectionManager,
    key: &str,
    config: &RateLimitConfig,
) -> Result<bool, redis::RedisError> {
    use redis::AsyncCommands;

    let mut conn = redis.clone();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs_f64();

    let window_start = now - config.window_secs as f64;

    // 1. Remove entries older than the window
    let _: () = conn
        .zrembyscore(key, f64::NEG_INFINITY, window_start)
        .await?;

    // 2. Count entries in the current window
    let count: u32 = conn.zcard(key).await?;

    if count >= config.max_requests {
        return Ok(false);
    }

    // 3. Add the current request timestamp
    let _: () = conn.zadd(key, now, now.to_string()).await?;

    // 4. Set TTL so the key self-cleans
    let _: () = conn.expire(key, config.window_secs as i64).await?;

    Ok(true)
}

// ---------------------------------------------------------------------------
// Axum-compatible middleware layer
// ---------------------------------------------------------------------------

/// Create a rate-limiting middleware closure for use with [`axum::middleware::from_fn`].
///
/// The returned closure captures the Redis connection and config, and applies
/// the sliding-window rate limiter to every request passing through the layer.
pub fn make_rate_limit_layer(
    redis: Arc<ConnectionManager>,
    config: RateLimitConfig,
    trusted_proxies: Arc<Vec<IpAddr>>,
    endpoint_group: String,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Clone
       + Send {
    move |req: Request, next: Next| {
        let redis = redis.clone();
        let config = config.clone();
        let trusted_proxies = trusted_proxies.clone();
        let endpoint_group = endpoint_group.clone();
        Box::pin(async move {
            rate_limit_middleware(
                redis,
                config,
                trusted_proxies,
                "default".to_string(),
                endpoint_group,
                req,
                next,
            )
            .await
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http;

    // -- Client IP extraction helpers --

    fn make_request_with_forwarded_for(value: &str) -> Request {
        let mut req = Request::builder()
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();
        req.headers_mut().insert(
            "x-forwarded-for",
            http::HeaderValue::from_str(value).unwrap(),
        );
        req
    }

    #[test]
    fn extract_client_ip_ignores_forwarded_for_without_trusted_proxy() {
        // No trusted proxies → X-Forwarded-For is ignored, falls back to peer (None here)
        let req = make_request_with_forwarded_for("203.0.113.50, 70.41.3.18, 150.172.238.178");
        let ip = extract_client_ip(&req, &[]);
        assert!(ip.is_none());
    }

    #[test]
    fn extract_client_ip_direct_no_header() {
        // No X-Forwarded-For and no ConnectInfo → None
        let req = Request::builder()
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();
        let ip = extract_client_ip(&req, &[]);
        assert!(ip.is_none());
    }

    #[test]
    fn extract_client_ip_invalid_forwarded_for_falls_through() {
        let req = make_request_with_forwarded_for("not-an-ip, also-not");
        let ip = extract_client_ip(&req, &[]);
        assert!(ip.is_none());
    }

    #[test]
    fn untrusted_proxy_ignores_forwarded_for() {
        // Request with X-Forwarded-For but no ConnectInfo and empty trusted list → None
        let req = make_request_with_forwarded_for("203.0.113.50");
        let ip = extract_client_ip(&req, &[]);
        assert!(ip.is_none());
    }

    #[test]
    fn trusted_proxy_uses_forwarded_for() {
        // Simulate trusted proxy by injecting ConnectInfo
        let mut req = make_request_with_forwarded_for("203.0.113.50, 10.0.0.1");
        let proxy_ip: IpAddr = "10.0.0.1".parse().unwrap();
        let sock_addr: std::net::SocketAddr = "10.0.0.1:12345".parse().unwrap();
        req.extensions_mut()
            .insert(axum::extract::ConnectInfo(sock_addr));

        let ip = extract_client_ip(&req, &[proxy_ip]);
        assert_eq!(ip, Some("203.0.113.50".parse().unwrap()));
    }

    #[test]
    fn no_proxies_configured_uses_peer_ip() {
        // ConnectInfo present but no trusted proxies → use peer IP directly
        let mut req = make_request_with_forwarded_for("203.0.113.50");
        let sock_addr: std::net::SocketAddr = "192.168.1.1:54321".parse().unwrap();
        req.extensions_mut()
            .insert(axum::extract::ConnectInfo(sock_addr));

        let ip = extract_client_ip(&req, &[]);
        assert_eq!(ip, Some("192.168.1.1".parse().unwrap()));
    }

    // -- Rate limit key format --

    #[test]
    fn rate_limit_key_format() {
        let key = build_rate_limit_key("proj_123", "10.0.0.1", "auth");
        assert_eq!(key, "rate:proj_123:10.0.0.1:auth");
    }

    #[test]
    fn rate_limit_key_format_api() {
        let key = build_rate_limit_key("proj_abc", "192.168.1.1", "api");
        assert_eq!(key, "rate:proj_abc:192.168.1.1:api");
    }

    // -- Config presets --

    #[test]
    fn config_stores_values() {
        let cfg = RateLimitConfig {
            max_requests: 120,
            window_secs: 30,
        };
        assert_eq!(cfg.max_requests, 120);
        assert_eq!(cfg.window_secs, 30);
    }
}
