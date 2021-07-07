use std::num::TryFromIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("TryFromInt error:{0}")]
    IntConversion(#[from] TryFromIntError),
}

pub type Result<T> = std::result::Result<T, Error>;
