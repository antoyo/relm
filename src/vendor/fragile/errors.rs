use std::error;
use std::fmt;

/// Returned when borrowing fails.
#[derive(Debug)]
pub struct InvalidThreadAccess;

impl fmt::Display for InvalidThreadAccess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(error::Error::description(self), f)
    }
}

impl error::Error for InvalidThreadAccess {
    fn description(&self) -> &str {
        "fragile value accessed from foreign thread"
    }
}
