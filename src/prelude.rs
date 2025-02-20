#![allow(unused)]
use crate::error::Error;
use oxc::ast::ast::{CallExpression, ImportDeclarationSpecifier, ImportOrExportKind, JSXAttribute, Statement, WithClause};
use oxc::ast::AstBuilder;
use oxc::allocator::{Box as OxcBox, Vec as OxcVec, IntoIn}; 
use oxc::span::{Atom, SPAN};


pub type Result<T> = core::result::Result<T, Error>;

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
