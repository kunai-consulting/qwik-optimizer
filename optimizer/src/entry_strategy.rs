use crate::component::{SegmentData, SegmentKind};
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
    fn get_entry_for_sym(&self, context: &[String], segment: &SegmentData) -> Option<String> {
        context.first().map_or_else(
            || Some(ENTRY_SEGMENTS.to_string()),
            |root| Some([segment.origin.display().to_string().as_str(), "_entry_", root].concat()),
        )
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
    fn get_entry_for_sym(&self, context: &[String], segment: &SegmentData) -> Option<String> {
        // Event handlers without scope variables are put into a separate file
        if segment.scoped_idents.is_empty()
            && (segment.ctx_kind != SegmentKind::Function || segment.ctx_name == "event$")
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
