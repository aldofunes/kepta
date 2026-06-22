use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("email is invalid")]
    InvalidEmail,
    #[error("password must be at least 8 characters")]
    PasswordTooShort,
    #[error("{field} is required")]
    Required { field: &'static str },
    #[error("{field} exceeds maximum length of {max}")]
    TooLong { field: &'static str, max: usize },
}

pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if email.contains('@') && email.len() <= 254 {
        Ok(())
    } else {
        Err(ValidationError::InvalidEmail)
    }
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() >= 8 {
        Ok(())
    } else {
        Err(ValidationError::PasswordTooShort)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_email_accepted() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("user+tag@sub.domain.io").is_ok());
    }

    #[test]
    fn email_without_at_rejected() {
        assert!(matches!(validate_email("notanemail"), Err(ValidationError::InvalidEmail)));
    }

    #[test]
    fn email_over_254_chars_rejected() {
        let long = format!("{}@b.com", "a".repeat(250));
        assert!(matches!(validate_email(&long), Err(ValidationError::InvalidEmail)));
    }

    #[test]
    fn password_exactly_8_chars_accepted() {
        assert!(validate_password("12345678").is_ok());
    }

    #[test]
    fn password_under_8_chars_rejected() {
        assert!(matches!(
            validate_password("short"),
            Err(ValidationError::PasswordTooShort)
        ));
    }

    #[test]
    fn empty_password_rejected() {
        assert!(matches!(
            validate_password(""),
            Err(ValidationError::PasswordTooShort)
        ));
    }
}
