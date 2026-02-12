use axum::http::HeaderValue;

const COOKIE_NAME: &str = "pinas_session";

/// Build a Set-Cookie header value for authentication
pub fn build_auth_cookie(token: &str, max_age_hours: u64, secure: bool) -> HeaderValue {
    let max_age_secs = max_age_hours * 3600;
    let mut cookie = format!(
        "{}={}; HttpOnly; SameSite=Strict; Path=/api; Max-Age={}",
        COOKIE_NAME, token, max_age_secs
    );
    if secure {
        cookie.push_str("; Secure");
    }
    // safe: token is base64-encoded JWT, no special chars
    HeaderValue::from_str(&cookie).unwrap_or_else(|_| HeaderValue::from_static(""))
}

/// Build a Set-Cookie header value that clears the auth cookie
pub fn build_clear_cookie(secure: bool) -> HeaderValue {
    let mut cookie = format!(
        "{}=; HttpOnly; SameSite=Strict; Path=/api; Max-Age=0",
        COOKIE_NAME
    );
    if secure {
        cookie.push_str("; Secure");
    }
    HeaderValue::from_str(&cookie).unwrap_or_else(|_| HeaderValue::from_static(""))
}

/// Extract the pinas_session token from a Cookie header value
pub fn extract_token_from_cookies(cookie_header: &str) -> Option<&str> {
    for part in cookie_header.split(';') {
        let trimmed = part.trim();
        if let Some(value) = trimmed.strip_prefix("pinas_session=") {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_auth_cookie_no_secure() {
        let cookie = build_auth_cookie("abc123", 24, false);
        let s = cookie.to_str().unwrap();
        assert!(s.contains("pinas_session=abc123"));
        assert!(s.contains("HttpOnly"));
        assert!(s.contains("SameSite=Strict"));
        assert!(s.contains("Path=/api"));
        assert!(s.contains("Max-Age=86400"));
        assert!(!s.contains("Secure"));
    }

    #[test]
    fn test_build_auth_cookie_secure() {
        let cookie = build_auth_cookie("abc123", 24, true);
        let s = cookie.to_str().unwrap();
        assert!(s.contains("; Secure"));
    }

    #[test]
    fn test_build_clear_cookie() {
        let cookie = build_clear_cookie(true);
        let s = cookie.to_str().unwrap();
        assert!(s.contains("pinas_session="));
        assert!(s.contains("Max-Age=0"));
        assert!(s.contains("; Secure"));
    }

    #[test]
    fn test_extract_token() {
        assert_eq!(
            extract_token_from_cookies("pinas_session=abc123"),
            Some("abc123")
        );
        assert_eq!(
            extract_token_from_cookies("other=xyz; pinas_session=abc123; another=def"),
            Some("abc123")
        );
        assert_eq!(
            extract_token_from_cookies("other=xyz"),
            None
        );
        assert_eq!(
            extract_token_from_cookies("pinas_session="),
            None
        );
    }
}
