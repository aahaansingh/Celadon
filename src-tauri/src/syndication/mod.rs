use chrono::{DateTime, Utc};
use std::error;
use std::fmt;

pub mod syndicator;

#[derive(Debug, Clone)]
struct RetrievalError;

impl fmt::Display for RetrievalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not retrieve this feed from the requested url")
    }
}

impl error::Error for RetrievalError {}

pub fn unwrap_default<T>(opt: Option<T>, default: T) -> T {
    match opt {
        None => default,
        Some(val) => val,
    }
}

pub fn unwrap_date(opt: Option<String>) -> DateTime<Utc> {
    match opt {
        None => Utc::now(),
        Some(timestamp) => {
            let parse_result = DateTime::parse_from_rfc2822(&timestamp);
            match parse_result {
                Err(e) => Utc::now(),
                Ok(dt) => dt.to_utc(),
            }
        }
    }
}
