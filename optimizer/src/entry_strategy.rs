use crate::component::SegmentData;
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

/// Trait for determining entry points for QRL segments.
///
/// Entry strategies control how QRL segments are grouped into separate files.
/// The `context` parameter is the `stack_ctxt` from TransformGenerator, containing
/// the component hierarchy (names of variable declarations, functions, classes,
/// JSX elements, and attributes encountered during traversal).
pub trait EntryPolicy: Send + Sync {
    /// Determines the entry file name for a segment.
    ///
    /// # Arguments
    /// * `context` - The stack_ctxt containing the component hierarchy
    /// * `segment` - The SegmentData with all QRL metadata (scoped_idents, ctx_kind, etc.)
    ///
    /// # Returns
    /// * `Some(entry_name)` - Group this segment with the given entry name
    /// * `None` - This segment gets its own file (no grouping)
    fn get_entry_for_sym(&self, context: &[String], segment: &SegmentData) -> Option<String>;
}

/// Inline all QRLs into the same entry point.
/// Used by EntryStrategy::Inline and EntryStrategy::Hoist.
#[derive(Default, Clone)]
pub struct InlineStrategy;

impl EntryPolicy for InlineStrategy {
    fn get_entry_for_sym(&self, _context: &[String], _segment: &SegmentData) -> Option<String> {
        Some(ENTRY_SEGMENTS.to_string())
    }
}

/// Put all QRLs into a single entry point.
/// Used by EntryStrategy::Single.
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

/// Each QRL segment gets its own file.
/// Used by EntryStrategy::Segment and EntryStrategy::Hook.
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

/// Group QRLs by their root component.
/// All QRLs within the same component share an entry point.
/// Used by EntryStrategy::Component.
#[derive(Clone)]
pub struct PerComponentStrategy {}

impl PerComponentStrategy {
    pub const fn new() -> Self {
        Self {}
    }
}

impl EntryPolicy for PerComponentStrategy {
    fn get_entry_for_sym(&self, _context: &[String], _segment: &SegmentData) -> Option<String> {
        // TODO: Implement in plan 07-02
        // Will use context.first() to get root component name
        // and segment.origin for the file origin
        panic!("PerComponentStrategy not yet implemented - see plan 07-02")
    }
}

/// Smart grouping: event handlers without captures get their own files,
/// other QRLs are grouped by component.
/// Used by EntryStrategy::Smart.
#[derive(Clone)]
pub struct SmartStrategy {}

impl SmartStrategy {
    pub const fn new() -> Self {
        Self {}
    }
}

impl EntryPolicy for SmartStrategy {
    fn get_entry_for_sym(&self, _context: &[String], _segment: &SegmentData) -> Option<String> {
        // TODO: Implement in plan 07-02
        // Will check segment.scoped_idents.is_empty() and segment.ctx_kind
        // Event handlers without captures get None (own file)
        // Others use context.first() for component grouping
        panic!("SmartStrategy not yet implemented - see plan 07-02")
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
