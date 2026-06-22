use anyhow::Result;
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use kepta_types::auth::TokenClaims;
use uuid::Uuid;

const ACCESS_TOKEN_SECS: i64 = 15 * 60;

pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtKeys {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn sign(&self, user_id: Uuid, email: &str) -> Result<String> {
        let now = Utc::now().timestamp();
        let claims = TokenClaims {
            sub: user_id.to_string(),
            email: email.to_string(),
            iat: now,
            exp: now + ACCESS_TOKEN_SECS,
        };
        let token = encode(&Header::default(), &claims, &self.encoding)?;
        Ok(token)
    }

    pub fn verify(&self, token: &str) -> Result<TokenClaims> {
        let data = decode::<TokenClaims>(token, &self.decoding, &Validation::new(Algorithm::HS256))?;
        Ok(data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> JwtKeys {
        JwtKeys::new("test-secret-that-is-long-enough-for-hs256")
    }

    #[test]
    fn sign_and_verify_round_trip() {
        let keys = keys();
        let id = Uuid::now_v7();
        let token = keys.sign(id, "user@example.com").unwrap();
        let claims = keys.verify(&token).unwrap();
        assert_eq!(claims.sub, id.to_string());
        assert_eq!(claims.email, "user@example.com");
    }

    #[test]
    fn exp_is_in_the_future() {
        let keys = keys();
        let token = keys.sign(Uuid::now_v7(), "u@e.com").unwrap();
        let claims = keys.verify(&token).unwrap();
        assert!(claims.exp > Utc::now().timestamp());
    }

    #[test]
    fn wrong_secret_rejected() {
        let token = keys().sign(Uuid::now_v7(), "u@e.com").unwrap();
        let other_keys = JwtKeys::new("completely-different-secret-padded-ok");
        assert!(other_keys.verify(&token).is_err());
    }

    #[test]
    fn garbage_token_rejected() {
        assert!(keys().verify("not.a.token").is_err());
    }

    #[test]
    fn tampered_payload_rejected() {
        let keys = keys();
        let token = keys.sign(Uuid::now_v7(), "u@e.com").unwrap();
        let mut parts: Vec<&str> = token.splitn(3, '.').collect();
        // Flip one char in the payload segment
        let mut payload = parts[1].to_string();
        let tampered = if payload.starts_with('e') { payload.replacen('e', "a", 1) }
                       else { payload.replacen('a', 'e'.to_string().as_str(), 1) };
        parts[1] = Box::leak(tampered.into_boxed_str());
        let bad_token = parts.join(".");
        assert!(keys.verify(&bad_token).is_err());
    }
}
