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
// Trusted proxy allowlist
// ---------------------------------------------------------------------------

/// A parsed list of trusted proxy CIDRs.
///
/// Only requests whose immediate peer address falls within one of these ranges
/// are permitted to supply an `X-Forwarded-For` header that overrides the
/// apparent client IP.  All other requests use the raw socket address.
#[derive(Debug, Clone, Default)]
pub struct TrustedProxies {
    cidrs: Vec<(IpAddr, u8)>,
}

impl TrustedProxies {
    /// Build from a slice of CIDR strings (e.g. `"10.0.0.0/8"`, `"127.0.0.1"`).
    ///
    /// Invalid entries are silently skipped.
    pub fn from_cidrs(cidrs: &[String]) -> Self {
        let parsed = cidrs
            .iter()
            .filter_map(|s| parse_cidr(s.trim()))
            .collect();
        Self { cidrs: parsed }
    }

    /// Returns `true` if `ip` falls within any trusted proxy range.
    pub fn contains(&self, ip: IpAddr) -> bool {
        self.cidrs.iter().any(|&(net, prefix_len)| ip_in_cidr(ip, net, prefix_len))
    }
}

/// Parse a single CIDR string into `(network_address, prefix_len)`.
///
/// A bare IP with no `/` prefix is treated as a host route (`/32` or `/128`).
fn parse_cidr(s: &str) -> Option<(IpAddr, u8)> {
    if let Some((addr_str, prefix_str)) = s.split_once('/') {
        let addr: IpAddr = addr_str.trim().parse().ok()?;
        let prefix_len: u8 = prefix_str.trim().parse().ok()?;
        Some((addr, prefix_len))
    } else {
        let addr: IpAddr = s.parse().ok()?;
        let prefix_len = match addr {
            IpAddr::V4(_) => 32,
            IpAddr::V6(_) => 128,
        };
        Some((addr, prefix_len))
    }
}

/// Returns `true` if `ip` is contained within the network `net/prefix_len`.
fn ip_in_cidr(ip: IpAddr, net: IpAddr, prefix_len: u8) -> bool {
    match (ip, net) {
        (IpAddr::V4(ip4), IpAddr::V4(net4)) => {
            if prefix_len == 0 {
                return true;
            }
            if prefix_len > 32 {
                return false;
            }
            let shift = 32 - u32::from(prefix_len);
            u32::from(ip4) >> shift == u32::from(net4) >> shift
        }
        (IpAddr::V6(ip6), IpAddr::V6(net6)) => {
            if prefix_len == 0 {
                return true;
            }
            if prefix_len > 128 {
                return false;
            }
            let shift = 128 - u128::from(prefix_len);
            u128::from(ip6) >> shift == u128::from(net6) >> shift
        }
        // Mismatched address families are never equal.
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Client IP extraction
// ---------------------------------------------------------------------------

/// Extract the real client IP address, respecting the trusted proxy allowlist.
///
/// * If `peer_addr` is `Some` **and** falls within `trusted_proxies`, the
///   left-most IP in the `X-Forwarded-For` header is returned (if present and
///   parseable).
/// * Otherwise the raw `peer_addr` is returned directly, ignoring any
///   `X-Forwarded-For` header that could have been injected by an attacker.
pub fn extract_client_ip(
    req: &Request,
    peer_addr: Option<IpAddr>,
    trusted_proxies: &TrustedProxies,
) -> Option<IpAddr> {
    if let Some(peer) = peer_addr {
        if trusted_proxies.contains(peer) {
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

    // Fall back to the connected peer address.
    peer_addr
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
    trusted_proxies: Arc<TrustedProxies>,
    project_id: String,
    endpoint_group: String,
    req: Request,
    next: Next,
) -> Response {
    let peer_addr = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip());

    let ip = extract_client_ip(&req, peer_addr, &trusted_proxies)
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
/// The returned closure captures the Redis connection, rate-limit config, and
/// trusted proxy list, applying the sliding-window rate limiter to every
/// request passing through the layer.
pub fn make_rate_limit_layer(
    redis: Arc<ConnectionManager>,
    config: RateLimitConfig,
    trusted_proxies: Arc<TrustedProxies>,
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
    use std::net::Ipv4Addr;

    // -- Helpers --

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

    fn no_proxies() -> TrustedProxies {
        TrustedProxies::default()
    }

    fn proxies(cidrs: &[&str]) -> TrustedProxies {
        TrustedProxies::from_cidrs(
            &cidrs.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        )
    }

    // -- TrustedProxies / CIDR matching --

    #[test]
    fn trusted_proxies_exact_ip() {
        let tp = proxies(&["10.0.0.1"]);
        assert!(tp.contains("10.0.0.1".parse().unwrap()));
        assert!(!tp.contains("10.0.0.2".parse().unwrap()));
    }

    #[test]
    fn trusted_proxies_cidr_ipv4() {
        let tp = proxies(&["10.0.0.0/8"]);
        assert!(tp.contains("10.1.2.3".parse().unwrap()));
        assert!(!tp.contains("11.0.0.1".parse().unwrap()));
    }

    #[test]
    fn trusted_proxies_cidr_slash32() {
        let tp = proxies(&["192.168.1.100/32"]);
        assert!(tp.contains("192.168.1.100".parse().unwrap()));
        assert!(!tp.contains("192.168.1.101".parse().unwrap()));
    }

    #[test]
    fn trusted_proxies_cidr_slash0() {
        // /0 matches everything
        let tp = proxies(&["0.0.0.0/0"]);
        assert!(tp.contains(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))));
    }

    #[test]
    fn trusted_proxies_invalid_cidr_skipped() {
        let tp = proxies(&["not-a-cidr", "10.0.0.0/8"]);
        // "not-a-cidr" is skipped, valid entry still works
        assert!(tp.contains("10.5.5.5".parse().unwrap()));
    }

    #[test]
    fn trusted_proxies_empty() {
        let tp = no_proxies();
        assert!(!tp.contains("10.0.0.1".parse().unwrap()));
    }

    // -- Client IP extraction --

    #[test]
    fn xff_trusted_when_peer_is_in_allowlist() {
        let req = make_request_with_forwarded_for("203.0.113.50, 70.41.3.18");
        let peer: IpAddr = "10.0.0.1".parse().unwrap();
        let tp = proxies(&["10.0.0.0/24"]);
        let ip = extract_client_ip(&req, Some(peer), &tp);
        assert_eq!(ip, Some("203.0.113.50".parse().unwrap()));
    }

    #[test]
    fn xff_ignored_when_peer_not_in_allowlist() {
        // Even though X-Forwarded-For claims a routable IP, the peer is not
        // a trusted proxy, so we use the raw peer address instead.
        let req = make_request_with_forwarded_for("203.0.113.50");
        let peer: IpAddr = "1.2.3.4".parse().unwrap();
        let ip = extract_client_ip(&req, Some(peer), &no_proxies());
        assert_eq!(ip, Some("1.2.3.4".parse().unwrap()));
    }

    #[test]
    fn xff_ignored_when_no_peer() {
        // No TCP peer at all (unit test scenario) → return None, do not use XFF.
        let req = make_request_with_forwarded_for("203.0.113.50");
        let ip = extract_client_ip(&req, None, &no_proxies());
        assert!(ip.is_none());
    }

    #[test]
    fn xff_invalid_value_falls_back_to_peer() {
        let req = make_request_with_forwarded_for("not-an-ip, also-not");
        let peer: IpAddr = "10.0.0.1".parse().unwrap();
        let tp = proxies(&["10.0.0.0/24"]);
        // Peer is trusted, but the XFF value is unparseable → fall back to peer.
        let ip = extract_client_ip(&req, Some(peer), &tp);
        assert_eq!(ip, Some(peer));
    }

    #[test]
    fn direct_connection_no_header() {
        let req = Request::builder()
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();
        let ip = extract_client_ip(&req, None, &no_proxies());
        assert!(ip.is_none());
    }

    #[test]
    fn direct_connection_with_peer() {
        let req = Request::builder()
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();
        let peer: IpAddr = "172.16.0.5".parse().unwrap();
        let ip = extract_client_ip(&req, Some(peer), &no_proxies());
        assert_eq!(ip, Some(peer));
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
