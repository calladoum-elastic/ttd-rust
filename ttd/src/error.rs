use derive_more::{Display, From};

/// The result type specific to this crate
pub type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
#[derive(From, Debug, Display, Default)]
pub enum Error {
    #[default]
    Unknown,

    Generic(String),

    MissingValue,
    NotFound,
    DataMismatch,
    InitializationError,
    ConversionError,
    ForeignFunctionError,

    #[from]
    Utf8Error(std::str::Utf8Error),

    #[from]
    Utf16Error(std::string::FromUtf16Error),

    #[from]
    #[allow(clippy::enum_variant_names)]
    WindowsError(windows::core::Error),

    #[from]
    Null(std::ffi::NulError),

    #[from]
    ParseError(std::num::ParseIntError),

    #[from]
    LoggerError(log::SetLoggerError),
}

impl std::error::Error for Error {}
