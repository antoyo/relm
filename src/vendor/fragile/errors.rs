use std::error;
use std::fmt;

/// Returned when borrowing fails.
#[derive(Debug)]
pub struct InvalidThreadAccess;

impl fmt::Display for InvalidThreadAccess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.to_string(), f)
    }
}

impl error::Error for InvalidThreadAccess {
    fn description(&self) -> &str {
        "fragile value accessed from foreign thread"
    }
}
