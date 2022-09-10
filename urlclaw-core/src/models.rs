use std::str::FromStr;

use thiserror::Error;
use url::Url;
use uuid::Uuid;

use crate::UrlclawError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortUrl {
    id: Uuid,

    short: Short,
    target: Url,
}

impl ShortUrl {
    pub fn new(short: String, target: Url) -> Result<Self, UrlclawError> {
        Ok(Self {
            id: Uuid::new_v4(),
            short: short.parse()?,
            target,
        })
    }

    pub fn from_db(id: Uuid, short: String, target: String) -> Result<Self, UrlclawError> {
        Ok(Self {
            id,
            short: short.parse()?,
            target: target.parse()?,
        })
    }

    pub fn uuid(&self) -> Uuid {
        self.id
    }

    pub fn short_url(&self) -> &Short {
        &self.short
    }

    pub fn target_url(&self) -> &Url {
        &self.target
    }
}

fn check_short_is_safe(short: &str) -> bool {
    short.chars().all(|c| Short::ALLOWED_CHARACTERS.contains(c))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum InvalidShortError {
    #[error("The short contains invalid chacaters.")]
    InvalidCharacters,
    #[error(
        "The short is shorter than the minimum of {0} characters",
        Short::MIN_LENGTH
    )]
    TooShort,
    #[error("The short exceeds {0} characters", Short::MAX_LENGTH)]
    TooLong,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Short(String);

/// Newtype for a Short-URL. Ensures Short-URLs only contain certain characters
/// and are kept in their length constraints.
impl Short {
    /// Minimum short length
    pub const MIN_LENGTH: usize = 1;

    /// Maximum short length
    pub const MAX_LENGTH: usize = 64;

    /// allowed characters allowed in short urls,  according to RFC 3986 section 2.3
    pub const ALLOWED_CHARACTERS: &'static str =
        "abcdefghjkilmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~";

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<Short> for String {
    fn from(s: Short) -> Self {
        s.0
    }
}

impl FromStr for Short {
    type Err = InvalidShortError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > Self::MAX_LENGTH {
            Err(InvalidShortError::TooLong)
        } else if s.len() < Self::MIN_LENGTH {
            Err(InvalidShortError::TooShort)
        } else if !check_short_is_safe(s) {
            Err(InvalidShortError::InvalidCharacters)
        } else {
            Ok(Self(s.to_owned()))
        }
    }
}

impl TryFrom<&str> for Short {
    type Error = InvalidShortError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_safety_check() {
        assert!(check_short_is_safe("rusty"));
        assert!(check_short_is_safe("Rusty42"));
        assert!(check_short_is_safe("Rusty"));
        assert!(check_short_is_safe("1234"));
        assert!(!check_short_is_safe("rusty!"));
        assert!(check_short_is_safe("~ferris~"));
        assert!(!check_short_is_safe("ferris is cool"));
        assert!(!check_short_is_safe("cool/stuff"));
        assert!(!check_short_is_safe("a?b"));
        assert!(!check_short_is_safe("a&b"));
        assert!(!check_short_is_safe("a#b"));
        assert!(!check_short_is_safe("a%20"));
    }

    #[test]
    fn test_short_fromstr() {
        assert_eq!("".parse::<Short>(), Err(InvalidShortError::TooShort));

        assert!("a".repeat(Short::MAX_LENGTH).parse::<Short>().is_ok());
        assert_eq!(
            "a".repeat(Short::MAX_LENGTH + 1).parse::<Short>(),
            Err(InvalidShortError::TooLong)
        );

        assert_eq!(
            "a?b".parse::<Short>(),
            Err(InvalidShortError::InvalidCharacters)
        );
    }
}
