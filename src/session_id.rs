use std::str::FromStr;
use std::error::Error;
use std::fmt::{Formatter, Display};

#[derive(Debug, Eq, PartialEq)]
pub enum SessionIdParseError {
    NonEmptyString,
}

impl Display for SessionIdParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Session Id must be a non-empty string")
    }
}

impl Error for SessionIdParseError {}

#[derive(Debug, Eq, PartialEq)]
pub struct SessionId {
    session_id: String,
}

impl FromStr for SessionId {
    type Err = SessionIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err(SessionIdParseError::NonEmptyString)
        } else {
            Ok(SessionId {
                session_id: s.to_owned(),
            })
        }
    }
}

impl Display for SessionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.session_id)
    }
}
