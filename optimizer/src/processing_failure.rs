use crate::illegal_code::IllegalCodeType;
use serde::Serialize;

/// Processing failure diagnostic following SWC format.
/// Used to report errors without failing the transformation.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize)]
pub struct ProcessingFailure {
    /// Error category - always "error" for illegal code
    pub category: String,
    /// Error code - "C02" for illegal code references
    pub code: String,
    /// Source file path
    pub file: String,
    /// Human-readable error message
    pub message: String,
    /// Scope where error occurred - always "optimizer"
    pub scope: String,
}

impl ProcessingFailure {
    /// Create a new illegal code diagnostic matching SWC format.
    ///
    /// # Arguments
    /// * `illegal_type` - The type of illegal code (function or class)
    /// * `file` - Source file path
    ///
    /// # Returns
    /// ProcessingFailure with C02 code and proper message format
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
