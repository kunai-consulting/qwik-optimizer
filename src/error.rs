use crate::illegal_code::IllegalCodeType;

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

    #[error( "Reference to identifier '{id}' can not be used inside a Qrl($) scope because it's a {expr_type}", id =.0.identifier(), expr_type = .0.expression_type())]
    IllegalCode(IllegalCodeType),
}
