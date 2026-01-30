//! Event handler transformation utilities for JSX.
//!
//! This module contains utilities for transforming Qwik JSX event handler
//! attribute names to HTML attribute format.
//!
//! # Functions
//!
//! - `get_event_scope_data_from_jsx_event`: Extract scope prefix and event name index
//! - `create_event_name`: Convert camelCase to kebab-case event names
//! - `jsx_event_to_html_attribute`: Transform onClick$ to on:click format

/// Extracts scope prefix and event name start index from a JSX event attribute name.
///
/// # Returns
/// A tuple of (prefix, start_index) where:
/// - prefix: "on:", "on-document:", or "on-window:"
/// - start_index: index where the event name begins (after "on", "document:on", or "window:on")
/// - If not an event, returns ("", usize::MAX)
///
/// # Examples
/// - "onClick$" -> ("on:", 2)
/// - "document:onFocus$" -> ("on-document:", 11)
/// - "window:onClick$" -> ("on-window:", 9)
/// - "custom$" -> ("", usize::MAX)
pub(crate) fn get_event_scope_data_from_jsx_event(jsx_event: &str) -> (&'static str, usize) {
    if jsx_event.starts_with("window:on") {
        ("on-window:", 9)
    } else if jsx_event.starts_with("document:on") {
        ("on-document:", 11)
    } else if jsx_event.starts_with("on") {
        ("on:", 2)
    } else {
        ("", usize::MAX)
    }
}

/// Creates an HTML event attribute name from an event name and prefix.
///
/// Converts camelCase to kebab-case (e.g., "Click" -> "click", "DblClick" -> "dblclick").
/// The `-` prefix in the original name preserves case (e.g., "-cLick" -> "c-lick").
///
/// # Examples
/// - ("Click", "on:") -> "on:click"
/// - ("DblClick", "on:") -> "on:dblclick"
/// - ("Focus", "on-document:") -> "on-document:focus"
pub(super) fn create_event_name(name: &str, prefix: &str) -> String {
    let mut result = String::from(prefix);

    // Check if name starts with '-' (case-preserving marker)
    let name = if let Some(stripped) = name.strip_prefix('-') {
        // Case-preserving: don't lowercase, but still convert camelCase humps to dashes
        for c in stripped.chars() {
            if c.is_ascii_uppercase() {
                result.push('-');
                result.push(c.to_ascii_lowercase());
            } else {
                result.push(c);
            }
        }
        return result;
    } else {
        name
    };

    // Standard camelCase to kebab-case: lowercase everything
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

/// Transforms a Qwik JSX event attribute name to HTML attribute format.
///
/// Returns `None` if the attribute is not a valid event (doesn't end with '$'
/// or doesn't start with a valid event prefix).
///
/// # Examples
/// - "onClick$" -> Some("on:click")
/// - "onDblClick$" -> Some("on:dblclick")
/// - "document:onFocus$" -> Some("on-document:focus")
/// - "window:onClick$" -> Some("on-window:click")
/// - "on-cLick$" -> Some("on:c-lick") (case preserved due to '-' prefix)
/// - "onClick" -> None (no '$' suffix)
/// - "custom$" -> None (not an event)
pub(crate) fn jsx_event_to_html_attribute(jsx_event: &str) -> Option<String> {
    // Must end with '$' to be a Qwik event handler
    if !jsx_event.ends_with('$') {
        return None;
    }

    let (prefix, idx) = get_event_scope_data_from_jsx_event(jsx_event);
    if idx == usize::MAX {
        return None;
    }

    // Extract event name: strip '$' suffix and take from idx
    // e.g., "onClick$" with idx=2 -> "Click"
    let name = &jsx_event[idx..jsx_event.len() - 1];

    Some(create_event_name(name, prefix))
}
