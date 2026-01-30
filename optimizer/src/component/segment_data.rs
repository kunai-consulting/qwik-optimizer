//! Segment metadata for QRL code generation.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::collector::{ExportInfo, Id};

/// Context type of a QRL segment (Function, EventHandler, or JSXProp).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum SegmentKind {
    Function,
    EventHandler,
    JSXProp,
}

impl Default for SegmentKind {
    fn default() -> Self {
        SegmentKind::Function
    }
}

impl SegmentKind {
    pub fn from_ctx_name(ctx_name: &str) -> Self {
        if ctx_name.starts_with("on") {
            if let Some(third_char) = ctx_name.chars().nth(2) {
                if third_char.is_ascii_uppercase() {
                    return SegmentKind::EventHandler;
                }
            }
        }
        SegmentKind::Function
    }

    pub fn event_handler() -> Self {
        SegmentKind::EventHandler
    }

    pub fn jsx_prop() -> Self {
        SegmentKind::JSXProp
    }

    pub fn is_event_handler(&self) -> bool {
        matches!(self, SegmentKind::EventHandler)
    }

    pub fn is_jsx_prop(&self) -> bool {
        matches!(self, SegmentKind::JSXProp)
    }
}

/// Complete metadata for a QRL segment file generation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct SegmentData {
    pub extension: String,
    pub local_idents: Vec<Id>,
    pub scoped_idents: Vec<Id>,
    pub parent_segment: Option<String>,
    pub ctx_kind: SegmentKind,
    pub ctx_name: String,
    pub origin: PathBuf,
    pub path: PathBuf,
    pub display_name: String,
    pub hash: String,
    pub need_transform: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub referenced_exports: Vec<ExportInfo>,
}

impl SegmentData {
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

    pub fn with_extension(mut self, extension: impl Into<String>) -> Self {
        self.extension = extension.into();
        self
    }

    pub fn has_captures(&self) -> bool {
        !self.scoped_idents.is_empty()
    }

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
