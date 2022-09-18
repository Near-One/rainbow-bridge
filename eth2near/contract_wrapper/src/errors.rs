use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct TryToSubmitZeroHeaderError;

impl Display for TryToSubmitZeroHeaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Try to submit zero headers")
    }
}

impl Error for TryToSubmitZeroHeaderError {}
