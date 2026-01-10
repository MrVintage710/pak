use thiserror::Error;

use crate::query::pql::PqlToken;

pub type PakResult<T> = Result<T, PakError>;

#[derive(Error, Debug)]
pub enum PakError {
    #[error("The source of the Pak is currenty being used by a seporate opperation.")]
    SourceInUse,
    #[error("Type mismatch error: {0} found, {1} expected")]
    TypeMismatchError(String, String),
    #[error("Was unable to update rules item: {0}")]
    UpdateRuleItemError(String),
    #[error("Was unable to insert rules item: {0}")]
    InsertRuleItemError(String),
    #[error("There was an error packing the module: {0}")]
    BincodeError(#[from] Box<bincode::ErrorKind>),
    #[error("{0}")]
    FileError(#[from] std::io::Error),
    #[error("PQL Error")]
    PqlError(#[from] PqlError),
    #[error("Unable to find index `{0}` in Pak. Make sure that this index is correctly spelled.")]
    InvalidIndex(String),
    #[error("Before interacting with a Pak file, identifier match check failed. This pointer must come from another file, or might be the wrong version.")]
    PakIdentifierMismatch
}

pub type PqlResult<T> = Result<T, PqlError>;

#[derive(Error, Debug)]
pub enum PqlError {
    #[error("Reached end of file unexpectedly at the end of a group.")]
    EndOfFile,
    #[error("")]
    NoMatch,
    #[error("Unexpected Token: Got `{0:?}` while expecting '{1}`.")]
    UnexpectedToken(PqlToken, String)
}

impl PqlError {
    pub fn is_no_match(&self) -> bool {
        matches!(self, PqlError::NoMatch)
    }
}