use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn hash(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("hashing failed: {e}"))?
        .to_string();
    Ok(hash)
}

pub fn verify(password: &str, hash: &str) -> Result<bool> {
    let parsed = PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("invalid hash: {e}"))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_correct_password() {
        let h = hash("correct-horse-battery").unwrap();
        assert!(verify("correct-horse-battery", &h).unwrap());
    }

    #[test]
    fn wrong_password_does_not_verify() {
        let h = hash("correct-horse-battery").unwrap();
        assert!(!verify("wrong-password", &h).unwrap());
    }

    #[test]
    fn two_hashes_of_same_password_are_distinct() {
        let h1 = hash("password").unwrap();
        let h2 = hash("password").unwrap();
        // Different salts → different hashes, but both verify
        assert_ne!(h1, h2);
        assert!(verify("password", &h1).unwrap());
        assert!(verify("password", &h2).unwrap());
    }

    #[test]
    fn garbage_hash_returns_error() {
        assert!(verify("password", "not-a-valid-hash").is_err());
    }
}
