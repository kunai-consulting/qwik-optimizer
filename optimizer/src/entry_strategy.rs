use crate::segment::Segment;
use serde::{Deserialize, Serialize};

const ENTRY_SEGMENTS: &str = "entry_segments";

// EntryStrategies
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

pub trait EntryPolicy: Send + Sync {
    fn get_entry_for_sym(&self, context: &[String], segment: &Segment) -> Option<String>;
}

#[derive(Default, Clone)]
pub struct InlineStrategy;

impl EntryPolicy for InlineStrategy {
    fn get_entry_for_sym(&self, _context: &[String], _segment: &Segment) -> Option<String> {
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
    fn get_entry_for_sym(&self, _context: &[String], _segment: &Segment) -> Option<String> {
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
    fn get_entry_for_sym(&self, _context: &[String], _segment: &Segment) -> Option<String> {
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
    fn get_entry_for_sym(&self, context: &[String], segment: &Segment) -> Option<String> {
        panic!("Not implemented")
        /*
        context.first().map_or_else(
            || Some(ENTRY_SEGMENTS.to_string()),
            |root| Some([&segment.origin, "_entry_", root].concat()),
        )
        */
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
    fn get_entry_for_sym(&self, context: &[String], segment: &Segment) -> Option<String> {
        panic!("Not implemented")
        /*
        // Event handlers without scope variables are put into a separate file
        if segment.scoped_idents.is_empty()
            && (segment.ctx_kind != SegmentKind::Function || &segment.ctx_name == "event$")
        {
            return None;
        }

        // Everything else is put into a single file per component
        // This means that all QRLs for a component are loaded together
        // if one is used
        context.first().map_or_else(
            // Top-level QRLs are put into a separate file
            || None,
            // Other QRLs are put into a file named after the original file + the root component
            |root| Some([&segment.origin, "_entry_", root].concat()),
        )
        */
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
