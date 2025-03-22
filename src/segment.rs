use crate::component::*;
use oxc_allocator::{Allocator, Box as OxcBox, FromIn};
use oxc_ast::ast::{BindingIdentifier, BindingPattern, BindingPatternKind, TSTypeAnnotation};
use oxc_ast::AstBuilder;
use oxc_span::SPAN;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Segment {
    Named(String),
    /// Represents a named capture with the `$` suffix e.g. `component$`, `foo$`.
    /// The `usize` is the number of times the prefix appears within the same segment scope and
    /// allows for the creation of unique names.
    NamedQrl(String, usize),
    /// Represents a segment that has been made unique by adding an index .e.g `_1`, `_2`, etc..
    /// This is only used for case where an unanchored QRL's, `$`, name needs to be made unique.
    IndexQrl(usize),
}

enum UniqueName {
    Name(String, usize),
    Index(usize),
}

pub(crate) struct SegmentBuilder {
    names: HashMap<String, usize>,
}

fn make_fq_name(segment_names: &Vec<String>) -> String {
    let mut fq_name = String::new();

    for segment in segment_names {
        if segment.trim().is_empty() {
            continue;
        }

        if fq_name.is_empty()
            && segment
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            fq_name = format!("_{}", segment);
        } else {
            let prefix: String = if fq_name.is_empty() {
                "".to_string()
            } else {
                format!("{}_", fq_name).to_string()
            };
            fq_name = format!("{}{}", prefix, segment);
        }
    }

    fq_name
}

fn make_unique_segment_name(
    existing_segment_names: Vec<String>,
    new_segment_name: &SegmentName,
    names: &mut HashMap<String, usize>,
) -> UniqueName {
    let mut existing_segment_names = existing_segment_names;

    if !matches!(new_segment_name, SegmentName::Name(_)) {
        if let SegmentName::AnchoredQrl(ref name) = new_segment_name {
            existing_segment_names.push(name.clone());
        }
        let fq_name = make_fq_name(&existing_segment_names);

        match names.get_mut(fq_name.as_str()) {
            None => match new_segment_name {
                SegmentName::AnchoredQrl(name) => {
                    names.insert(fq_name, 1);
                    UniqueName::Name(name.clone(), 0)
                }
                SegmentName::UnanchoredQrl => {
                    names.insert(fq_name, 1);
                    UniqueName::Index(0)
                }
                SegmentName::Name(_) => unreachable!(),
            },
            Some(count) => match new_segment_name {
                SegmentName::AnchoredQrl(name) => {
                    let name = UniqueName::Name(name.clone(), count.clone());
                    *count += 1;
                    name
                }
                SegmentName::UnanchoredQrl => {
                    let name = UniqueName::Index(count.clone());
                    *count += 1;
                    name
                }
                SegmentName::Name(_) => unreachable!(),
            },
        }
    } else {
        match &new_segment_name {
            SegmentName::Name(name) => UniqueName::Name(name.clone(), 0),
            SegmentName::AnchoredQrl(_) => unreachable!(),
            SegmentName::UnanchoredQrl => unreachable!(),
        }
    }
}

enum SegmentName {
    Name(String),
    AnchoredQrl(String),
    UnanchoredQrl,
}

impl SegmentName {
    fn new(name0: String) -> Self {
        let name = name0.strip_suffix(MARKER_SUFFIX);

        match name {
            None => SegmentName::Name(name0),
            Some(name) if name.is_empty() => SegmentName::UnanchoredQrl,
            Some(name) => SegmentName::AnchoredQrl(name.to_string()),
        }
    }

    fn is_qrl(&self) -> bool {
        match self {
            SegmentName::AnchoredQrl(_) => true,
            SegmentName::UnanchoredQrl => true,
            SegmentName::Name(_) => false,
        }
    }
}

impl SegmentBuilder {
    pub(crate) fn new() -> Self {
        SegmentBuilder {
            names: HashMap::new(),
        }
    }

    pub fn new_segment<T: AsRef<str>>(&mut self, input: T, segments: &[Segment]) -> Segment {
        let input = input.as_ref();
        let segment_name = SegmentName::new(input.to_string());

        let segment_names: Vec<String> = segments.iter().map(|s| s.into()).collect();

        let unique_name = make_unique_segment_name(segment_names, &segment_name, &mut self.names);

        match unique_name {
            UniqueName::Name(name, index) => {
                if segment_name.is_qrl() {
                    Segment::NamedQrl(name, index)
                } else {
                    Segment::Named(name)
                }
            }
            UniqueName::Index(index) => Segment::IndexQrl(index),
        }
    }
}

impl Segment {
    pub fn is_qrl(&self) -> bool {
        match self {
            Segment::Named(_) => false,
            Segment::NamedQrl(_, _) => true,
            Segment::IndexQrl(_) => true,
        }
    }

    pub fn qrl_type(&self) -> Option<QrlType> {
        match self {
            Segment::Named(_) => None,
            Segment::NamedQrl(name, _) => Some(QrlType::PrefixedQrl(name.into())),
            Segment::IndexQrl(index) => Some(QrlType::IndexedQrl(*index)),
        }
    }

    fn into_binding_identifier<'a>(&self, allocator: &'a Allocator) -> BindingIdentifier<'a> {
        let ast_builder = AstBuilder::new(allocator);
        match self {
            Segment::Named(name) => ast_builder.binding_identifier(SPAN, name),
            Segment::NamedQrl(name, _) => {
                ast_builder.binding_identifier(SPAN, format!("{}{}", name, MARKER_SUFFIX))
            }
            Segment::IndexQrl(_) => ast_builder.binding_identifier(SPAN, MARKER_SUFFIX),
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
            Segment::Named(name) => write!(f, "{name}"),
            Segment::NamedQrl(name, index) if *index == 0 => write!(f, "{name}"),
            Segment::NamedQrl(name, index) => write!(f, "{name}_{index}"),
            Segment::IndexQrl(index) => write!(f, "${index}"),
        }
    }
}

impl<'a> FromIn<'a, Segment> for BindingPattern<'a> {
    fn from_in(value: Segment, allocator: &'a Allocator) -> Self {
        value.into_binding_pattern(allocator)
    }
}

impl From<&Segment> for String {
    fn from(input: &Segment) -> Self {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_fq_name() {
        let segments = vec!["foo".to_string(), "bar".to_string()];
        let fq_name = make_fq_name(&segments);
        assert_eq!(fq_name, "foo_bar");
    }

    #[test]
    fn test_new_segment_unique_name_for_qrl() {
        let mut builder = SegmentBuilder::new();
        let segments = vec![Segment::Named("foo".to_string())];
        let segment = builder.new_segment("bar$", &segments);
        assert_eq!(segment, Segment::NamedQrl("bar".to_string(), 0));

        let segment = builder.new_segment("bar$", &segments);
        assert_eq!(segment, Segment::NamedQrl("bar".to_string(), 1));
    }

    #[test]
    fn test_new_segment_unique_name_for_anonymous_qrl() {
        let mut builder = SegmentBuilder::new();
        let segments = vec![Segment::Named("foo".to_string())];
        let segment = builder.new_segment("$", &segments);
        assert_eq!(segment, Segment::IndexQrl(0));

        let segment = builder.new_segment("$", &segments);
        assert_eq!(segment, Segment::IndexQrl(1));
    }

    #[test]
    fn test_non_unique_for_non_qrl() {
        let mut builder = SegmentBuilder::new();
        let segments = vec![Segment::Named("foo".to_string())];
        let segment = builder.new_segment("bar", &segments);
        assert_eq!(segment, Segment::Named("bar".to_string()));

        let segment = builder.new_segment("bar", &segments);
        assert_eq!(segment, Segment::Named("bar".to_string()));
    }
}
