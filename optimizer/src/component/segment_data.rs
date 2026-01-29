//! Segment metadata for QRL code generation.
//!
//! This module provides `SegmentData` which captures the complete context of a QRL
//! extraction - captured variables, parent segment, context kind, and all other
//! metadata needed to generate segment files with proper imports and useLexicalScope
//! injection.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::collector::{ExportInfo, Id};

/// Indicates the context type of a QRL segment.
///
/// This enum distinguishes between different QRL usage contexts, which affects
/// how the segment is processed and optimized.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum SegmentKind {
    /// Regular function QRL (e.g., `$(() => ...)`)
    Function,
    /// Event handler QRL (e.g., `onClick$(() => ...)`)
    EventHandler,
    /// JSX prop QRL (e.g., `prop$={...}`)
    JSXProp,
}

impl Default for SegmentKind {
    fn default() -> Self {
        SegmentKind::Function
    }
}

impl SegmentKind {
    /// Determines the segment kind from a context name.
    ///
    /// - Names starting with "on" (case-sensitive) are treated as EventHandlers
    /// - Otherwise treated as Function (JSXProp is determined by call site context)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use qwik_optimizer::component::SegmentKind;
    ///
    /// assert_eq!(SegmentKind::from_ctx_name("onClick$"), SegmentKind::EventHandler);
    /// assert_eq!(SegmentKind::from_ctx_name("onInput$"), SegmentKind::EventHandler);
    /// assert_eq!(SegmentKind::from_ctx_name("component$"), SegmentKind::Function);
    /// assert_eq!(SegmentKind::from_ctx_name("$"), SegmentKind::Function);
    /// ```
    pub fn from_ctx_name(ctx_name: &str) -> Self {
        // Event handlers start with "on" followed by uppercase letter
        // e.g., onClick$, onInput$, onKeyDown$
        if ctx_name.starts_with("on") {
            // Check if the third character is uppercase (indicating an event handler)
            // This distinguishes "onClick" from hypothetical "once$" or similar
            if let Some(third_char) = ctx_name.chars().nth(2) {
                if third_char.is_ascii_uppercase() {
                    return SegmentKind::EventHandler;
                }
            }
        }
        SegmentKind::Function
    }

    /// Creates an EventHandler variant.
    pub fn event_handler() -> Self {
        SegmentKind::EventHandler
    }

    /// Creates a JSXProp variant.
    pub fn jsx_prop() -> Self {
        SegmentKind::JSXProp
    }

    /// Returns true if this is an event handler segment.
    pub fn is_event_handler(&self) -> bool {
        matches!(self, SegmentKind::EventHandler)
    }

    /// Returns true if this is a JSX prop segment.
    pub fn is_jsx_prop(&self) -> bool {
        matches!(self, SegmentKind::JSXProp)
    }
}

/// Complete metadata for a QRL segment.
///
/// This struct captures all the information needed to generate a segment file,
/// including captured variables for `useLexicalScope` injection, context kind,
/// file paths, and identifiers.
///
/// Matches SWC's `SegmentData` structure for compatibility.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct SegmentData {
    /// File extension (e.g., "js", "ts", "tsx")
    pub extension: String,

    /// All identifiers used within the segment (for import generation)
    pub local_idents: Vec<Id>,

    /// Identifiers captured from enclosing scope (for useLexicalScope)
    pub scoped_idents: Vec<Id>,

    /// Parent segment name for nested QRLs (None if top-level)
    pub parent_segment: Option<String>,

    /// Context kind (Function, EventHandler, JSXProp)
    pub ctx_kind: SegmentKind,

    /// Context name (e.g., "onClick$", "component$")
    pub ctx_name: String,

    /// Origin file path (relative)
    pub origin: PathBuf,

    /// Directory path
    pub path: PathBuf,

    /// Human-readable display name
    pub display_name: String,

    /// Hash string for segment identification
    pub hash: String,

    /// Whether the segment needs transformation (useLexicalScope injection)
    pub need_transform: bool,

    /// Source file exports referenced in the QRL body.
    /// These need to be imported in the segment file from the source file.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub referenced_exports: Vec<ExportInfo>,
}

impl SegmentData {
    /// Creates a new SegmentData with all required fields.
    ///
    /// # Arguments
    ///
    /// * `ctx_name` - The context name (e.g., "onClick$", "component$")
    /// * `display_name` - Human-readable name for the segment
    /// * `hash` - Unique hash identifying this segment
    /// * `origin` - Source file path
    /// * `scoped_idents` - Variables captured from enclosing scope
    /// * `local_idents` - Variables used locally in the segment
    /// * `parent_segment` - Parent segment name if this is nested
    pub fn new(
        ctx_name: &str,
        display_name: String,
        hash: String,
        origin: PathBuf,
        scoped_idents: Vec<Id>,
        local_idents: Vec<Id>,
        parent_segment: Option<String>,
    ) -> Self {
        Self::new_with_exports(
            ctx_name,
            display_name,
            hash,
            origin,
            scoped_idents,
            local_idents,
            parent_segment,
            Vec::new(),
        )
    }

    /// Creates a new SegmentData with referenced exports for segment file import generation.
    ///
    /// # Arguments
    ///
    /// * `ctx_name` - The context name (e.g., "onClick$", "component$")
    /// * `display_name` - Human-readable name for the segment
    /// * `hash` - Unique hash identifying this segment
    /// * `origin` - Source file path
    /// * `scoped_idents` - Variables captured from enclosing scope
    /// * `local_idents` - Variables used locally in the segment
    /// * `parent_segment` - Parent segment name if this is nested
    /// * `referenced_exports` - Source file exports referenced in QRL body
    pub fn new_with_exports(
        ctx_name: &str,
        display_name: String,
        hash: String,
        origin: PathBuf,
        scoped_idents: Vec<Id>,
        local_idents: Vec<Id>,
        parent_segment: Option<String>,
        referenced_exports: Vec<ExportInfo>,
    ) -> Self {
        let ctx_kind = SegmentKind::from_ctx_name(ctx_name);
        let need_transform = !scoped_idents.is_empty();

        Self {
            extension: "js".to_string(),
            local_idents,
            scoped_idents,
            parent_segment,
            ctx_kind,
            ctx_name: ctx_name.to_string(),
            origin: origin.clone(),
            path: origin.parent().unwrap_or(&origin).to_path_buf(),
            display_name,
            hash,
            need_transform,
            referenced_exports,
        }
    }

    /// Creates a new SegmentData with a specified extension.
    pub fn with_extension(mut self, extension: impl Into<String>) -> Self {
        self.extension = extension.into();
        self
    }

    /// Returns true if this segment has captured variables.
    ///
    /// Segments with captures need `useLexicalScope` injection to access
    /// variables from the enclosing scope.
    pub fn has_captures(&self) -> bool {
        !self.scoped_idents.is_empty()
    }

    /// Returns the number of captured variables.
    pub fn capture_count(&self) -> usize {
        self.scoped_idents.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_kind_default() {
        assert_eq!(SegmentKind::default(), SegmentKind::Function);
    }

    #[test]
    fn test_from_ctx_name_event_handlers() {
        assert_eq!(SegmentKind::from_ctx_name("onClick$"), SegmentKind::EventHandler);
        assert_eq!(SegmentKind::from_ctx_name("onInput$"), SegmentKind::EventHandler);
        assert_eq!(SegmentKind::from_ctx_name("onKeyDown$"), SegmentKind::EventHandler);
        assert_eq!(SegmentKind::from_ctx_name("onMouseEnter$"), SegmentKind::EventHandler);
    }

    #[test]
    fn test_from_ctx_name_functions() {
        assert_eq!(SegmentKind::from_ctx_name("component$"), SegmentKind::Function);
        assert_eq!(SegmentKind::from_ctx_name("$"), SegmentKind::Function);
        assert_eq!(SegmentKind::from_ctx_name("useTask$"), SegmentKind::Function);
        assert_eq!(SegmentKind::from_ctx_name("useVisibleTask$"), SegmentKind::Function);
        // "once$" should be Function, not EventHandler (no uppercase after "on")
        assert_eq!(SegmentKind::from_ctx_name("once$"), SegmentKind::Function);
    }

    #[test]
    fn test_segment_kind_helpers() {
        assert!(SegmentKind::EventHandler.is_event_handler());
        assert!(!SegmentKind::Function.is_event_handler());
        assert!(SegmentKind::JSXProp.is_jsx_prop());
        assert!(!SegmentKind::Function.is_jsx_prop());
    }

    #[test]
    fn test_segment_data_new() {
        use oxc_semantic::ScopeId;

        let scoped = vec![("count".to_string(), ScopeId::new(1))];
        let local = vec![("useState".to_string(), ScopeId::new(0))];

        let data = SegmentData::new(
            "onClick$",
            "Counter_onClick".to_string(),
            "abc123".to_string(),
            PathBuf::from("src/components/Counter.tsx"),
            scoped.clone(),
            local.clone(),
            None,
        );

        assert_eq!(data.ctx_name, "onClick$");
        assert_eq!(data.ctx_kind, SegmentKind::EventHandler);
        assert_eq!(data.display_name, "Counter_onClick");
        assert_eq!(data.hash, "abc123");
        assert_eq!(data.origin, PathBuf::from("src/components/Counter.tsx"));
        assert_eq!(data.path, PathBuf::from("src/components"));
        assert!(data.need_transform); // has scoped_idents
        assert!(data.has_captures());
        assert_eq!(data.capture_count(), 1);
    }

    #[test]
    fn test_segment_data_no_captures() {
        let data = SegmentData::new(
            "component$",
            "Counter_component".to_string(),
            "def456".to_string(),
            PathBuf::from("src/Counter.tsx"),
            vec![], // no captures
            vec![],
            None,
        );

        assert_eq!(data.ctx_kind, SegmentKind::Function);
        assert!(!data.need_transform); // no scoped_idents
        assert!(!data.has_captures());
        assert_eq!(data.capture_count(), 0);
    }

    #[test]
    fn test_segment_data_with_parent() {
        let data = SegmentData::new(
            "onClick$",
            "Counter_component_onClick".to_string(),
            "ghi789".to_string(),
            PathBuf::from("src/Counter.tsx"),
            vec![],
            vec![],
            Some("Counter_component".to_string()),
        );

        assert_eq!(data.parent_segment, Some("Counter_component".to_string()));
    }

    #[test]
    fn test_segment_data_with_extension() {
        let data = SegmentData::new(
            "$",
            "test".to_string(),
            "xyz".to_string(),
            PathBuf::from("test.ts"),
            vec![],
            vec![],
            None,
        )
        .with_extension("ts");

        assert_eq!(data.extension, "ts");
    }
}
