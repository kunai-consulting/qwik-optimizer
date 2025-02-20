use crate::error::Error;
use oxc::ast::ast::{CallExpression, JSXAttribute};
use std::borrow::Cow;

pub type Result<T> = core::result::Result<T, Error>;

/// Drop the last element of implementations container type.
pub trait DropLast {
    fn drop_last(&self) -> &Self;
}

// impl DropLast for String {
//     fn drop_last(&self) -> &Self {
//         let mut chars = self.chars().clone();
//         chars.next_back();
//         &chars.as_str().to_string()
//     }
// }

impl DropLast for str {
    fn drop_last(&self) -> &Self {
        self.strip_suffix(|_: char| true).unwrap_or(self)
    }
}

pub trait SegmentRef<'a> {
    fn normalize_name(&self) -> String;

    fn is_qwik(&self) -> bool;
}

impl<'a> SegmentRef<'a> for CallExpression<'a> {
    fn normalize_name(&self) -> String {
        let name = self.callee_name();
        let name= name.unwrap_or_default();
         name.strip_suffix("$").unwrap_or(name).to_string()
    }

    fn is_qwik(&self) -> bool {
        match self.callee_name() {
            Some(name) => name.ends_with("$"),
            None => false,
        }
    }
}

impl<'a> SegmentRef<'a> for JSXAttribute<'a> {
    fn normalize_name(&self) -> String {
        let name = self.name.get_identifier().name;
        name.strip_suffix("$").unwrap_or(name.as_str()).to_string()
    }

    fn is_qwik(&self) -> bool {
        self.name.get_identifier().name.ends_with("$")
    }
}
