pub mod feed_api;
use std::fmt;

#[derive(Debug, Clone)]
struct EmptyError;

impl fmt::Display for EmptyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No valid objects returned")
    }
}