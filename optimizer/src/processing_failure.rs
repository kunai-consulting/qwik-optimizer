use crate::illegal_code::IllegalCodeType;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum ProcessingFailure {
    IllegalCode(IllegalCodeType),
}

impl From<&IllegalCodeType> for ProcessingFailure {
    fn from(value: &IllegalCodeType) -> Self {
        ProcessingFailure::IllegalCode(value.clone())
    }
}
