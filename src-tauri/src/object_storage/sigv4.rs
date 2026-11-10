use chrono::{DateTime, Utc};
use ring::hmac;
use sha2::{Digest, Sha256};

use super::settings::Credentials;
use crate::foundation::app_error::{AppError, AppResult};

const ALGORITHM: &str = "AWS4-HMAC-SHA256";
const SERVICE: &str = "s3";

/// Headers returned in signing order; `host` and `x-amz-*` are set from the
/// signed values, so callers must send them verbatim.
pub(crate) fn authorization(
    method: &str,
    url: &reqwest::Url,
    region: &str,
    credentials: &Credentials,
    content_type: Option<&str>,
    payload: &[u8],
    now: DateTime<Utc>,
) -> AppResult<SignedHeaders> {
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = now.format("%Y%m%d").to_string();
    let payload_hash = hex::encode(Sha256::digest(payload));

    let canonical_headers = canonical_headers(url, content_type, &payload_hash, &amz_date);
    let signed_headers = canonical_headers.signed_headers();
    let canonical_request = format!(
        "{method}\n{}\n\n{}\n{signed_headers}\n{payload_hash}",
        url.path(),
        canonical_headers.render()
    );
    let scope = format!("{date_stamp}/{region}/{SERVICE}/aws4_request");
    let string_to_sign = format!(
        "{ALGORITHM}\n{amz_date}\n{scope}\n{}",
        hex::encode(Sha256::digest(canonical_request.as_bytes()))
    );
    let signature = hex::encode(hmac_digest(
        &signing_key(
            &credentials.secret_access_key,
            &date_stamp,
            region,
            SERVICE,
        ),
        string_to_sign.as_bytes(),
    ));
    Ok(SignedHeaders {
        authorization: format!(
            "{ALGORITHM} Credential={}/{scope}, SignedHeaders={signed_headers}, Signature={signature}",
            credentials.access_key_id
        ),
        amz_date,
        payload_hash,
        content_type: content_type.map(str::to_owned),
    })
}

pub(crate) struct SignedHeaders {
    pub authorization: String,
    pub amz_date: String,
    pub payload_hash: String,
    pub content_type: Option<String>,
}

struct CanonicalHeaders {
    content_type: Option<String>,
    host: String,
    payload_hash: String,
    amz_date: String,
}

fn canonical_headers(
    url: &reqwest::Url,
    content_type: Option<&str>,
    payload_hash: &str,
    amz_date: &str,
) -> CanonicalHeaders {
    CanonicalHeaders {
        content_type: content_type.map(str::to_owned),
        host: url
            .host_str()
            .expect("validated https URL has a host")
            .to_owned(),
        payload_hash: payload_hash.to_owned(),
        amz_date: amz_date.to_owned(),
    }
}

impl CanonicalHeaders {
    fn render(&self) -> String {
        let mut rendered = String::new();
        if let Some(content_type) = &self.content_type {
            rendered.push_str(&format!("content-type:{content_type}\n"));
        }
        rendered.push_str(&format!(
            "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
            self.host, self.payload_hash, self.amz_date
        ));
        rendered
    }

    fn signed_headers(&self) -> &'static str {
        if self.content_type.is_some() {
            "content-type;host;x-amz-content-sha256;x-amz-date"
        } else {
            "host;x-amz-content-sha256;x-amz-date"
        }
    }
}

fn signing_key(secret_access_key: &str, date_stamp: &str, region: &str, service: &str) -> hmac::Key {
    let mut key = hmac::Key::new(
        hmac::HMAC_SHA256,
        format!("AWS4{secret_access_key}").as_bytes(),
    );
    for component in [date_stamp, region, service, "aws4_request"] {
        let digest = hmac_digest(&key, component.as_bytes());
        key = hmac::Key::new(hmac::HMAC_SHA256, digest.as_ref());
    }
    key
}

fn hmac_digest(key: &hmac::Key, data: &[u8]) -> ring::hmac::Tag {
    hmac::sign(key, data)
}

pub(crate) fn ensure_signable_url(url: &reqwest::Url) -> AppResult<()> {
    // The canonical request signs the path verbatim; generated object keys are
    // percent-encoding free, and anything else would silently sign the wrong
    // resource.
    if url.path().bytes().any(|byte| {
        !(byte.is_ascii_alphanumeric() || b"/.-_".contains(&byte))
    }) || url.query().is_some()
    {
        return Err(AppError::invalid(
            "Object storage URLs may not contain a query or characters that need percent-encoding",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
