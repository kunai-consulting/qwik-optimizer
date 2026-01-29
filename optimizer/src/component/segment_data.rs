//! Segment metadata for QRL code generation.
//!
//! This module provides `SegmentData` which captures the complete context of a QRL
//! extraction - captured variables, parent segment, context kind, and all other
//! metadata needed to generate segment files with proper imports and useLexicalScope
//! injection.

use serde::{Deserialize, Serialize};

/// Indicates the context type of a QRL segment.
///
/// This enum distinguishes between different QRL usage contexts, which affects
/// how the segment is processed and optimized.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
    /// ```
    /// use optimizer::component::SegmentKind;
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
}
