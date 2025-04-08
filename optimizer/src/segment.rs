use crate::component::*;
use oxc_allocator::{Allocator, Box as OxcBox, FromIn};
use oxc_ast::ast::{BindingIdentifier, BindingPattern, BindingPatternKind, TSTypeAnnotation};
use oxc_ast::AstBuilder;
use oxc_span::SPAN;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Segment {
    Named(String),
    /// Represents a component captured by the `$` function.
    AnonymousCaptured,
    /// Represents a named capture with the `$` suffix e.g. `component$`, `foo$`.
    NamedCaptured(String),
}

impl Segment {
    fn new<T: AsRef<str>>(input: T) -> Segment {
        let input = input.as_ref();
        if input == MARKER_SUFFIX {
            Segment::AnonymousCaptured
        } else {
            match input.strip_suffix(MARKER_SUFFIX) {
                Some(name) => Segment::NamedCaptured(name.to_string()),
                None => Segment::Named(input.into()),
            }
        }
    }

    pub fn is_qrl_extractable(&self) -> bool {
        match self {
            Segment::Named(_) => false,
            Segment::AnonymousCaptured => true,
            Segment::NamedCaptured(_) => true,
        }
    }

    pub fn associated_qrl_type(&self) -> QrlType {
        match self {
            Segment::Named(_) => QrlType::Qrl,
            Segment::AnonymousCaptured => QrlType::Qrl,
            Segment::NamedCaptured(prefix) => QrlType::PrefixedQrl(prefix.into()),
        }
    }

    fn into_binding_identifier<'a>(&self, allocator: &'a Allocator) -> BindingIdentifier<'a> {
        let ast_builder = AstBuilder::new(allocator);
        match self {
            Segment::Named(name) => ast_builder.binding_identifier(SPAN, name),
            Segment::AnonymousCaptured => ast_builder.binding_identifier(SPAN, MARKER_SUFFIX),
            Segment::NamedCaptured(name) => {
                ast_builder.binding_identifier(SPAN, format!("{}{}", name, MARKER_SUFFIX))
            }
        }
    }

    fn into_binding_pattern<'a>(&self, allocator: &'a Allocator) -> BindingPattern<'a> {
        let ast_builder = AstBuilder::new(allocator);
        let id = OxcBox::new_in(self.into_binding_identifier(allocator), allocator);
        ast_builder.binding_pattern(
            BindingPatternKind::BindingIdentifier(id),
            None::<OxcBox<'a, TSTypeAnnotation<'a>>>,
            false,
        )
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::Named(name) => write!(f, "{}", name),
            Segment::AnonymousCaptured => write!(f, ""),
            Segment::NamedCaptured(name) => write!(f, "{}", name),
            // Segment::ComponentCaptured => write!(f, "{}", COMPONENT),
        }
    }
}

impl<'a> FromIn<'a, Segment> for BindingPattern<'a> {
    fn from_in(value: Segment, allocator: &'a Allocator) -> Self {
        value.into_binding_pattern(allocator)
    }
}

impl<'a> FromIn<'a, &'a BindingPattern<'a>> for Segment {
    fn from_in(value: &'a BindingPattern<'a>, _a: &'a Allocator) -> Self {
        let s: String = value
            .get_identifier_name()
            .iter()
            .map(|s| s.to_string())
            .collect();
        Segment::new(s)
    }
}

impl From<&Segment> for String {
    fn from(input: &Segment) -> Self {
        input.to_string()
    }
}

impl<T> From<T> for Segment
where
    T: AsRef<str>,
{
    fn from(input: T) -> Self {
        Segment::new(input)
    }
}
