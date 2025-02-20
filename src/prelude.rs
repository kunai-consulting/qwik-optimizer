use crate::error::Error;

pub type Result<T> = core::result::Result<T, Error>;

pub trait DropLast {
    fn drop_last(self) -> Self;
}

impl DropLast for String {
    fn drop_last(self) -> String {
        let mut chars = self.chars().clone();
        chars.next_back();
        chars.as_str().to_string()
    }
}
