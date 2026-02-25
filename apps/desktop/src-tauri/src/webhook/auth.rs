use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, PartialEq)]
pub enum AuthMode {
    None,
    Token(String),
    HmacSha256(String),
}

impl AuthMode {
    pub fn from_config(config: &serde_json::Value) -> Self {
        let mode = config.get("authMode").and_then(|v| v.as_str()).unwrap_or("none");
        match mode {
            "token" => {
                let token = config.get("authToken").and_then(|v| v.as_str()).unwrap_or("");
                AuthMode::Token(token.to_string())
            }
            "hmac" => {
                let secret = config.get("hmacSecret").and_then(|v| v.as_str()).unwrap_or("");
                AuthMode::HmacSha256(secret.to_string())
            }
            _ => AuthMode::None,
        }
    }
}

/// Validate an incoming request against the configured auth mode.
/// Returns Ok(()) if valid, Err(reason) if not.
pub fn validate_auth(
    mode: &AuthMode,
    authorization_header: Option<&str>,
    signature_header: Option<&str>,
    body: &[u8],
) -> Result<(), String> {
    match mode {
        AuthMode::None => Ok(()),
        AuthMode::Token(expected) => {
            let header = authorization_header
                .ok_or_else(|| "Missing Authorization header".to_string())?;
            let token = header.strip_prefix("Bearer ").unwrap_or(header);
            // Constant-time comparison
            if constant_time_eq(token.as_bytes(), expected.as_bytes()) {
                Ok(())
            } else {
                Err("Invalid token".to_string())
            }
        }
        AuthMode::HmacSha256(secret) => {
            let sig = signature_header
                .ok_or_else(|| "Missing X-Signature header".to_string())?;
            let expected_sig = compute_hmac(secret.as_bytes(), body);
            if constant_time_eq(sig.as_bytes(), expected_sig.as_bytes()) {
                Ok(())
            } else {
                Err("Invalid HMAC signature".to_string())
            }
        }
    }
}

fn compute_hmac(key: &[u8], data: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC can take key of any size");
    mac.update(data);
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

/// Constant-time byte comparison to prevent timing attacks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// Replace the hex crate dependency with our own hex_encode
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_none_always_passes() {
        let result = validate_auth(&AuthMode::None, None, None, b"anything");
        assert!(result.is_ok());
    }

    #[test]
    fn test_auth_token_valid() {
        let mode = AuthMode::Token("secret123".to_string());
        let result = validate_auth(&mode, Some("Bearer secret123"), None, b"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_auth_token_missing_header() {
        let mode = AuthMode::Token("secret123".to_string());
        let result = validate_auth(&mode, None, None, b"");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing"));
    }

    #[test]
    fn test_auth_token_wrong_value() {
        let mode = AuthMode::Token("secret123".to_string());
        let result = validate_auth(&mode, Some("Bearer wrong"), None, b"");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid token"));
    }

    #[test]
    fn test_auth_hmac_valid() {
        let secret = "my-secret";
        let body = b"hello world";
        let sig = compute_hmac(secret.as_bytes(), body);
        let mode = AuthMode::HmacSha256(secret.to_string());
        let result = validate_auth(&mode, None, Some(&sig), body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_auth_hmac_missing_signature() {
        let mode = AuthMode::HmacSha256("secret".to_string());
        let result = validate_auth(&mode, None, None, b"body");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing X-Signature"));
    }

    #[test]
    fn test_auth_hmac_wrong_signature() {
        let mode = AuthMode::HmacSha256("secret".to_string());
        let result = validate_auth(&mode, None, Some("deadbeef"), b"body");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid HMAC"));
    }
}
