use crate::illegal_code::IllegalCodeType;
use serde::Serialize;

/// Processing failure diagnostic for reporting errors without failing transformation.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize)]
pub struct ProcessingFailure {
    pub category: String,
    pub code: String,
    pub file: String,
    pub message: String,
    pub scope: String,
}

impl ProcessingFailure {
    pub fn illegal_code(illegal_type: &IllegalCodeType, file: &str) -> Self {
        let identifier = illegal_type.identifier();
        let expr_type = illegal_type.expression_type();

        ProcessingFailure {
            category: "error".to_string(),
            code: "C02".to_string(),
            file: file.to_string(),
            message: format!(
                "Reference to identifier '{}' can not be used inside a Qrl($) scope because it's a {}",
                identifier,
                expr_type
            ),
            scope: "optimizer".to_string(),
        }
    }
}

impl From<&IllegalCodeType> for ProcessingFailure {
    fn from(value: &IllegalCodeType) -> Self {
        // Default file when not known - caller should use illegal_code() with proper file
        ProcessingFailure::illegal_code(value, "unknown")
    }
}
