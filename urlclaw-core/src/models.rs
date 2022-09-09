use url::Url;
use uuid::Uuid;

/// allowed characters allowed in short urls,  according to RFC 3986 section 2.3
pub const ALLOWED_SHORT_CHARACTERS: &'static str =
    "abcdefghjkilmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortUrl {
    id: Uuid,

    short: String,
    target: Url,
}

impl ShortUrl {
    pub fn new(short: String, target: Url) -> Result<Self, ()> {
        if !check_short_is_safe(&short) {
            Err(())
        } else {
            Ok(Self {
                id: Uuid::new_v4(),
                short,
                target,
            })
        }
    }

    pub fn from_db(id: Uuid, short: String, target: String) -> Result<Self, ()> {
        Ok(Self {
            id,
            short: short.to_owned(),
            target: target.parse().unwrap(),
        })
    }

    pub fn uuid(&self) -> Uuid {
        self.id
    }

    pub fn short_url(&self) -> &str {
        &self.short
    }

    pub fn target_url(&self) -> &Url {
        &self.target
    }
}

pub fn check_short_is_safe(short: &str) -> bool {
    short.chars().all(|c| ALLOWED_SHORT_CHARACTERS.contains(c))
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
}
