use derive_more::{Display, From};

pub type Result<T> = core::result::Result<T, Error>;

#[allow(dead_code)]
#[derive(From, Debug, Display, Default)]
pub enum Error {
    #[default]
    Unknown,
    Generic(String),

    ForeignFunctionError,
    ConversionError,
    // #[from]
    // Utf8Error(core::str::Utf8Error),

    // #[from]
    // ParseError(core::num::ParseIntError),
}

impl core::error::Error for Error {}
