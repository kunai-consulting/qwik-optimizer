#![allow(unused)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic error: {0}")]
    Generic(String),


    #[error("Failed to convert OsStr, '{0}'. Context: {1}")]
    StringConversion(String, String),

    #[error("Unsupported language from SourceType: {0}")]
    UnsupportedLanguage(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),


    #[error(transparent)]
    OxcUnknownExtension(#[from] oxc_span::UnknownExtension),
}
