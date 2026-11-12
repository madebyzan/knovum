use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ring::hmac;

use crate::foundation::app_error::{AppError, AppResult};

/// Kling's API authenticates either with a single API key sent as the bearer
/// token, or with a legacy access/secret key pair stored as `ak.sk` that signs
/// a short-lived HS256 JWT. Neither key half contains a dot, so the separator
/// decides the scheme.
pub(super) fn bearer_token(secret: &str) -> AppResult<String> {
    match secret.split_once('.') {
        Some((access_key, signing_key)) if !access_key.is_empty() && !signing_key.is_empty() => {
            Ok(access_key_pair_token(access_key, signing_key)?)
        }
        Some(_) => Err(AppError::not_configured(
            "Kling credentials must be a single API key or an 'AccessKey.SecretKey' pair.",
        )),
        None => Ok(secret.to_string()),
    }
}

fn access_key_pair_token(access_key: &str, signing_key: &str) -> AppResult<String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| AppError::new("provider_request_failed", error.to_string()))?
        .as_secs() as i64;
    let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"HS256","typ":"JWT"}"#);
    let claims = serde_json::json!({ "iss": access_key, "exp": now + 1800, "nbf": now - 5 });
    let payload = URL_SAFE_NO_PAD.encode(claims.to_string());
    let signing_input = format!("{header}.{payload}");
    let key = hmac::Key::new(hmac::HMAC_SHA256, signing_key.as_bytes());
    let signature = hmac::sign(&key, signing_input.as_bytes());
    Ok(format!(
        "{signing_input}.{}",
        URL_SAFE_NO_PAD.encode(signature.as_ref())
    ))
}

#[cfg(test)]
#[path = "tests/auth.rs"]
mod tests;
