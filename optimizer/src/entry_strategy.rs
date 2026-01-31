use crate::component::{SegmentData, SegmentKind};
use serde::{Deserialize, Serialize};

const ENTRY_SEGMENTS: &str = "entry_segments";

#[derive(Debug, Serialize, Copy, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EntryStrategy {
    Inline,
    Hoist,
    Single,
    Hook,
    Segment,
    Component,
    Smart,
}

/// Determines entry points for QRL segment grouping into separate files.
pub trait EntryPolicy: Send + Sync {
    /// Returns Some(entry_name) to group with that entry, or None for own file.
    fn get_entry_for_sym(&self, context: &[String], segment: &SegmentData) -> Option<String>;
}

#[derive(Default, Clone)]
pub struct InlineStrategy;

impl EntryPolicy for InlineStrategy {
    fn get_entry_for_sym(&self, _context: &[String], _segment: &SegmentData) -> Option<String> {
        Some(ENTRY_SEGMENTS.to_string())
    }
}

#[derive(Clone)]
pub struct SingleStrategy {}

impl SingleStrategy {
    pub const fn new() -> Self {
        Self {}
    }
}

impl EntryPolicy for SingleStrategy {
    fn get_entry_for_sym(&self, _context: &[String], _segment: &SegmentData) -> Option<String> {
        Some(ENTRY_SEGMENTS.to_string())
    }
}

#[derive(Clone)]
pub struct PerSegmentStrategy {}

impl PerSegmentStrategy {
    pub const fn new() -> Self {
        Self {}
    }
}

impl EntryPolicy for PerSegmentStrategy {
    fn get_entry_for_sym(&self, _context: &[String], _segment: &SegmentData) -> Option<String> {
        None
    }
}

#[derive(Clone)]
pub struct PerComponentStrategy {}

impl PerComponentStrategy {
    pub const fn new() -> Self {
        Self {}
    }
}

impl EntryPolicy for PerComponentStrategy {
    fn get_entry_for_sym(&self, context: &[String], segment: &SegmentData) -> Option<String> {
        context.first().map_or_else(
            || Some(ENTRY_SEGMENTS.to_string()),
            |root| Some([segment.origin.display().to_string().as_str(), "_entry_", root].concat()),
        )
    }
}

#[derive(Clone)]
pub struct SmartStrategy {}

impl SmartStrategy {
    pub const fn new() -> Self {
        Self {}
    }
}

impl EntryPolicy for SmartStrategy {
    fn get_entry_for_sym(&self, context: &[String], segment: &SegmentData) -> Option<String> {
        if segment.scoped_idents.is_empty()
            && (segment.ctx_kind != SegmentKind::Function || segment.ctx_name == "event$")
        {
            return None;
        }

        context.first().map_or_else(
            || None,
            |root| Some([segment.origin.display().to_string().as_str(), "_entry_", root].concat()),
        )
    }
}

pub fn parse_entry_strategy(strategy: &EntryStrategy) -> Box<dyn EntryPolicy> {
    match strategy {
        EntryStrategy::Inline | EntryStrategy::Hoist => Box::<InlineStrategy>::default(),
        EntryStrategy::Hook => Box::new(PerSegmentStrategy::new()),
        EntryStrategy::Segment => Box::new(PerSegmentStrategy::new()),
        EntryStrategy::Single => Box::new(SingleStrategy::new()),
        EntryStrategy::Component => Box::new(PerComponentStrategy::new()),
        EntryStrategy::Smart => Box::new(SmartStrategy::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::SegmentData;
    use std::path::PathBuf;

    fn make_segment(ctx_name: &str, origin: &str, scoped_idents: Vec<(&str, u32)>) -> SegmentData {
        use oxc_semantic::ScopeId;
        let scoped: Vec<_> = scoped_idents
            .into_iter()
            .map(|(name, id)| (name.to_string(), ScopeId::new(id as usize)))
            .collect();
        SegmentData::new(
            ctx_name,
            "test_display".to_string(),
            "hash123".to_string(),
            PathBuf::from(origin),
            scoped,
            vec![],
            None,
        )
    }

    #[test]
    fn test_inline_strategy_always_returns_entry_segments() {
        let strategy = InlineStrategy;
        let segment = make_segment("component$", "test.tsx", vec![]);
        assert_eq!(
            strategy.get_entry_for_sym(&["Counter".into()], &segment),
            Some("entry_segments".to_string())
        );
    }

    #[test]
    fn test_inline_strategy_no_context() {
        let strategy = InlineStrategy;
        let segment = make_segment("$", "test.tsx", vec![]);
        assert_eq!(
            strategy.get_entry_for_sym(&[], &segment),
            Some("entry_segments".to_string())
        );
    }

    #[test]
    fn test_single_strategy_always_returns_entry_segments() {
        let strategy = SingleStrategy::new();
        let segment = make_segment("component$", "test.tsx", vec![]);
        assert_eq!(
            strategy.get_entry_for_sym(&["Counter".into()], &segment),
            Some("entry_segments".to_string())
        );
    }

    #[test]
    fn test_per_segment_strategy_always_returns_none() {
        let strategy = PerSegmentStrategy::new();
        let segment = make_segment("onClick$", "test.tsx", vec![]);
        assert_eq!(
            strategy.get_entry_for_sym(&["Counter".into()], &segment),
            None
        );
    }

    #[test]
    fn test_per_segment_strategy_no_context() {
        let strategy = PerSegmentStrategy::new();
        let segment = make_segment("$", "test.tsx", vec![]);
        assert_eq!(
            strategy.get_entry_for_sym(&[], &segment),
            None
        );
    }

    #[test]
    fn test_per_component_with_context() {
        let strategy = PerComponentStrategy::new();
        let segment = make_segment("onClick$", "src/Counter.tsx", vec![]);
        assert_eq!(
            strategy.get_entry_for_sym(&["Counter".into()], &segment),
            Some("src/Counter.tsx_entry_Counter".to_string())
        );
    }

    #[test]
    fn test_per_component_no_context() {
        let strategy = PerComponentStrategy::new();
        let segment = make_segment("$", "test.tsx", vec![]);
        assert_eq!(
            strategy.get_entry_for_sym(&[], &segment),
            Some("entry_segments".to_string())
        );
    }

    #[test]
    fn test_smart_event_handler_no_captures() {
        let strategy = SmartStrategy::new();
        let segment = make_segment("onClick$", "test.tsx", vec![]);
        assert_eq!(
            strategy.get_entry_for_sym(&["Counter".into()], &segment),
            None
        );
    }

    #[test]
    fn test_smart_event_handler_with_captures() {
        let strategy = SmartStrategy::new();
        let segment = make_segment("onClick$", "src/Counter.tsx", vec![("count", 1)]);
        assert_eq!(
            strategy.get_entry_for_sym(&["Counter".into()], &segment),
            Some("src/Counter.tsx_entry_Counter".to_string())
        );
    }

    #[test]
    fn test_smart_function_with_context() {
        let strategy = SmartStrategy::new();
        let segment = make_segment("component$", "src/Counter.tsx", vec![]);
        assert_eq!(
            strategy.get_entry_for_sym(&["Counter".into()], &segment),
            Some("src/Counter.tsx_entry_Counter".to_string())
        );
    }

    #[test]
    fn test_smart_no_context() {
        let strategy = SmartStrategy::new();
        let segment = make_segment("component$", "test.tsx", vec![]);
        assert_eq!(
            strategy.get_entry_for_sym(&[], &segment),
            None
        );
    }
}
