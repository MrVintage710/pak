use thiserror::Error;

pub type PakResult<T> = Result<T, PakError>;

#[derive(Error, Debug)]
pub enum PakError {
    #[error("Was unable to update rules item: {0}")]
    UpdateRuleItemError(String),
    #[error("Was unable to insert rules item: {0}")]
    InsertRuleItemError(String),
    #[error("There was an error packing the module: {0}")]
    BincodeError(#[from] Box<bincode::ErrorKind>),
    #[error("There was an error packing the module: {0}")]
    FileError(#[from] std::io::Error),
}