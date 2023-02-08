use std::fmt;

#[derive(Debug, Default)]
pub struct RateLimitError(pub ());

impl fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Rate limited")
    }
}

impl std::error::Error for RateLimitError {}