use anyhow::Result;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde_json::Value;

type HmacSha256 = Hmac<Sha256>;

pub fn sign_payload(secret: &str, body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(body);
    hex::encode(mac.finalize().into_bytes())
}

pub async fn dispatch(client: &reqwest::Client, url: &str, secret: &str, payload: &Value) -> Result<()> {
    let body = serde_json::to_vec(payload)?;
    let sig = sign_payload(secret, &body);

    client
        .post(url)
        .header("Content-Type", "application/json")
        .header("X-Kepta-Signature", format!("sha256={sig}"))
        .body(body)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn same_secret_and_body_produce_same_signature() {
        let body = b"hello world";
        let s1 = sign_payload("secret", body);
        let s2 = sign_payload("secret", body);
        assert_eq!(s1, s2);
    }

    #[test]
    fn different_secrets_produce_different_signatures() {
        let body = b"hello world";
        assert_ne!(sign_payload("secret-a", body), sign_payload("secret-b", body));
    }

    #[test]
    fn different_bodies_produce_different_signatures() {
        assert_ne!(sign_payload("s", b"body-a"), sign_payload("s", b"body-b"));
    }

    #[test]
    fn signature_is_lowercase_hex() {
        let sig = sign_payload("s", b"body");
        assert!(sig.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
        assert_eq!(sig.len(), 64); // SHA-256 → 32 bytes → 64 hex chars
    }

    #[test]
    fn json_payload_signature_is_deterministic() {
        let payload = json!({"event": "maintenance.due", "household_id": "abc"});
        let body = serde_json::to_vec(&payload).unwrap();
        let s1 = sign_payload("webhook-secret", &body);
        let s2 = sign_payload("webhook-secret", &body);
        assert_eq!(s1, s2);
    }
}
