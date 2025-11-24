use derive_more::{Display, From};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(From, Debug, Display, Default)]
pub enum Error {
    #[default]
    Unknown,
    Generic(String),

    #[from]
    Utf16Error(std::string::FromUtf16Error),

    ForeignFunctionError,
    ConversionError,
}

impl core::error::Error for Error {}
