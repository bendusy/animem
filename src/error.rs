use thiserror::Error;

pub type Result<T> = std::result::Result<T, AnimemError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AnimemError {
    #[error("input is empty")]
    EmptyInput,
    #[error("invalid range: start={start}, end={end}, len={len}")]
    InvalidRange {
        start: usize,
        end: usize,
        len: usize,
    },
}
