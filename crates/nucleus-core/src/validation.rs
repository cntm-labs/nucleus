use crate::error::AppError;
use std::net::IpAddr;
use url::Url;
use validator::ValidateEmail;

/// Validate and normalize an email address (lowercase, trim)
pub fn validate_email(email: &str) -> Result<String, AppError> {
    let trimmed = email.trim();
    let normalized = trimmed.to_lowercase();
    if !normalized.validate_email() {
        return Err(crate::error::UserError::InvalidEmail.into());
    }
    Ok(normalized)
}

/// Validate password meets requirements (min 8, max 128 chars)
pub fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(crate::error::AuthError::PasswordTooWeak.into());
    }
    if password.len() > 128 {
        return Err(crate::error::AuthError::PasswordTooWeak.into());
    }
    Ok(())
}

/// Check if an IP address is in a private/internal range
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            // 127.0.0.0/8
            octets[0] == 127
            // 10.0.0.0/8
            || octets[0] == 10
            // 172.16.0.0/12
            || (octets[0] == 172 && (octets[1] & 0xF0) == 16)
            // 192.168.0.0/16
            || (octets[0] == 192 && octets[1] == 168)
            // 169.254.0.0/16
            || (octets[0] == 169 && octets[1] == 254)
            // 0.0.0.0
            || (octets[0] == 0 && octets[1] == 0 && octets[2] == 0 && octets[3] == 0)
        }
        IpAddr::V6(v6) => {
            // ::1
            v6.is_loopback()
            // fc00::/7
            || (v6.segments()[0] & 0xFE00) == 0xFC00
        }
    }
}

/// Validate a URL and reject private/internal IP addresses (SSRF protection)
pub fn validate_webhook_url(url_str: &str) -> Result<Url, AppError> {
    let parsed = Url::parse(url_str)
        .map_err(|e| anyhow::anyhow!("invalid URL: {}", e))?;

    // Only allow http and https
    match parsed.scheme() {
        "http" | "https" => {}
        scheme => return Err(anyhow::anyhow!("unsupported URL scheme: {}", scheme).into()),
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("URL has no host"))?;

    // Reject localhost by name
    if host == "localhost" {
        return Err(anyhow::anyhow!("private/internal URLs are not allowed").into());
    }

    // Try to parse as IP directly
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_private_ip(&ip) {
            return Err(anyhow::anyhow!("private/internal URLs are not allowed").into());
        }
    } else {
        // Resolve hostname and check all resolved IPs
        use std::net::ToSocketAddrs;
        let port = parsed.port().unwrap_or(match parsed.scheme() {
            "https" => 443,
            _ => 80,
        });
        if let Ok(addrs) = (host, port).to_socket_addrs() {
            for addr in addrs {
                if is_private_ip(&addr.ip()) {
                    return Err(
                        anyhow::anyhow!("private/internal URLs are not allowed").into(),
                    );
                }
            }
        }
    }

    Ok(parsed)
}

/// Validate a slug (lowercase alphanumeric + hyphens, 3-100 chars)
pub fn validate_slug(slug: &str) -> Result<(), AppError> {
    if slug.len() < 3 || slug.len() > 100 {
        return Err(anyhow::anyhow!(
            "slug must be between 3 and 100 characters"
        )
        .into());
    }
    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(anyhow::anyhow!(
            "slug must only contain lowercase alphanumeric characters and hyphens"
        )
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_email_accepts_valid() {
        let result = validate_email("user@example.com");
        assert_eq!(result.unwrap(), "user@example.com");
    }

    #[test]
    fn validate_email_normalizes_case() {
        let result = validate_email("User@Example.COM");
        assert_eq!(result.unwrap(), "user@example.com");
    }

    #[test]
    fn validate_email_trims_whitespace() {
        let result = validate_email(" user@example.com ");
        assert_eq!(result.unwrap(), "user@example.com");
    }

    #[test]
    fn validate_email_rejects_invalid() {
        assert!(validate_email("not-an-email").is_err());
    }

    #[test]
    fn validate_password_accepts_valid() {
        assert!(validate_password("SecurePass123!").is_ok());
    }

    #[test]
    fn validate_password_rejects_too_short() {
        assert!(validate_password("abc").is_err());
    }

    #[test]
    fn validate_password_rejects_too_long() {
        let long = "a".repeat(129);
        assert!(validate_password(&long).is_err());
    }

    #[test]
    fn validate_webhook_url_accepts_valid() {
        let result = validate_webhook_url("https://example.com/webhook");
        assert!(result.is_ok());
    }

    #[test]
    fn validate_webhook_url_rejects_private_ip() {
        assert!(validate_webhook_url("http://192.168.1.1/webhook").is_err());
    }

    #[test]
    fn validate_webhook_url_rejects_localhost() {
        assert!(validate_webhook_url("http://localhost/webhook").is_err());
        assert!(validate_webhook_url("http://127.0.0.1/webhook").is_err());
    }

    #[test]
    fn validate_slug_accepts_valid() {
        assert!(validate_slug("my-project-123").is_ok());
    }

    #[test]
    fn validate_slug_rejects_uppercase() {
        assert!(validate_slug("MyProject").is_err());
    }

    #[test]
    fn validate_slug_rejects_special_chars() {
        assert!(validate_slug("my project!").is_err());
    }

    #[test]
    fn validate_slug_rejects_too_short() {
        assert!(validate_slug("ab").is_err());
    }
}
