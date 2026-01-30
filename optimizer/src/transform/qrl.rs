//! QRL extraction and segment management for Qwik optimizer.
//!
//! This module contains QRL-specific transformation logic extracted from
//! generator.rs following the dispatcher pattern. Contains functions for:
//!
//! - Scoped identifier computation (`compute_scoped_idents`)
//! - Display name generation
//! - Hash computation
//! - QRL extraction during call expression handling

use std::collections::HashSet;

use base64::{engine, Engine};
use std::hash::{DefaultHasher, Hasher};

use crate::collector::Id;

use super::generator::{IdentType, IdPlusType};

/// Compute which identifiers from parent scopes are captured by a QRL.
///
/// Takes all identifiers referenced in the QRL body and all declarations from parent scopes,
/// returning the intersection (sorted for deterministic output) and whether all captured
/// variables are const.
///
/// # Arguments
/// * `all_idents` - All identifiers referenced in the QRL body (from IdentCollector)
/// * `all_decl` - All declarations from parent scopes (flattened decl_stack)
///
/// # Returns
/// A tuple of:
/// * `Vec<Id>` - Sorted list of captured identifiers
/// * `bool` - True if all captured variables are const
pub fn compute_scoped_idents(all_idents: &[Id], all_decl: &[IdPlusType]) -> (Vec<Id>, bool) {
    let mut set: HashSet<Id> = HashSet::new();
    let mut is_const = true;

    for ident in all_idents {
        // Compare by name only - ScopeId differences between IdentCollector (uses 0)
        // and decl_stack (uses actual scope) should not prevent capture detection.
        // For QRL capture purposes, name matching is sufficient since we're comparing
        // within a single file's scope hierarchy.
        if let Some(item) = all_decl.iter().find(|item| item.0 .0 == ident.0) {
            // Use the declaration's full Id (with correct scope) rather than collector's Id
            set.insert(item.0.clone());
            if !matches!(item.1, IdentType::Var(true)) {
                is_const = false;
            }
        }
    }

    let mut output: Vec<Id> = set.into_iter().collect();
    output.sort(); // Deterministic ordering for stable output
    (output, is_const)
}

/// Builds the display name from a segment stack.
///
/// Joins segment names with underscores, handling special cases for named QRLs
/// and indexed QRLs.
pub(crate) fn build_display_name(segment_stack: &[crate::segment::Segment]) -> String {
    use crate::segment::Segment;

    let mut display_name = String::new();

    for segment in segment_stack {
        let segment_str: String = match segment {
            Segment::Named(name) => name.clone(),
            Segment::NamedQrl(name, 0) => name.clone(),
            Segment::NamedQrl(name, index) => format!("{}_{}", name, index),
            Segment::IndexQrl(0) => continue, // Skip zero-indexed QRLs
            Segment::IndexQrl(index) => index.to_string(),
        };

        if segment_str.is_empty() {
            continue;
        }

        if display_name.is_empty() {
            // Prefix with underscore if starts with digit
            if segment_str
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                display_name = format!("_{}", segment_str);
            } else {
                display_name = segment_str;
            }
        } else {
            display_name = format!("{}_{}", display_name, segment_str);
        }
    }

    display_name
}

/// Calculates a hash for a QRL given the source info, display name, and optional scope.
///
/// Uses the source file path, display name, and scope to generate a stable hash.
pub(crate) fn compute_hash(
    rel_path: &std::path::Path,
    display_name: &str,
    scope: Option<&str>,
) -> String {
    let local_file_name = rel_path.to_string_lossy();
    let normalized_local_file_name = local_file_name
        .strip_prefix("./")
        .unwrap_or(&local_file_name);

    let mut hasher = DefaultHasher::new();
    if let Some(scope) = scope {
        hasher.write(scope.as_bytes());
    }
    hasher.write(normalized_local_file_name.as_bytes());
    hasher.write(display_name.as_bytes());
    let hash = hasher.finish();

    engine::general_purpose::URL_SAFE_NO_PAD
        .encode(hash.to_le_bytes())
        .replace(['-', '_'], "0")
}

/// Collects imported identifier names from a list of imports.
///
/// Used to filter out identifiers that will be imported from those that need
/// to be captured via useLexicalScope.
///
/// # Arguments
/// * `imports` - List of imports to collect names from
///
/// # Returns
/// A HashSet of imported identifier names
pub(crate) fn collect_imported_names(imports: &[crate::component::Import]) -> HashSet<String> {
    use crate::component::ImportId;

    imports
        .iter()
        .flat_map(|import| import.names.iter())
        .filter_map(|id| match id {
            ImportId::Named(name) => Some(name.clone()),
            ImportId::Default(name) => Some(name.clone()),
            ImportId::NamedWithAlias(_, local) => Some(local.clone()),
            ImportId::Namespace(_) => None, // Namespace imports are accessed via member expr
        })
        .collect()
}

/// Filters scoped identifiers by removing those that are imported.
///
/// Identifiers that are imported should not be captured via useLexicalScope
/// since they will be handled via imports in the segment file.
///
/// # Arguments
/// * `scoped_idents` - List of captured identifiers
/// * `imported_names` - Set of imported identifier names to exclude
///
/// # Returns
/// Filtered list of captured identifiers
pub(crate) fn filter_imported_from_scoped(
    scoped_idents: Vec<Id>,
    imported_names: &HashSet<String>,
) -> Vec<Id> {
    scoped_idents
        .into_iter()
        .filter(|(name, _)| !imported_names.contains(name))
        .collect()
}

/// Collects referenced exports from identifiers in a QRL body.
///
/// Identifiers in the QRL body that are exports from the source file need
/// to be imported in segment files.
///
/// # Arguments
/// * `descendent_idents` - All identifiers used in the QRL body
/// * `imported_names` - Set of imported names (to exclude)
/// * `scoped_idents` - Set of captured variables (to exclude)
/// * `export_by_name` - Map of exports in the source file
///
/// # Returns
/// List of ExportInfo for referenced exports
pub(crate) fn collect_referenced_exports(
    descendent_idents: &[Id],
    imported_names: &HashSet<String>,
    scoped_idents: &[Id],
    export_by_name: &std::collections::HashMap<String, crate::collector::ExportInfo>,
) -> Vec<crate::collector::ExportInfo> {
    descendent_idents
        .iter()
        .filter_map(|(name, _)| {
            // Skip if it's an import (will be handled via imports)
            if imported_names.contains(name) {
                return None;
            }
            // Skip if it's a captured variable (handled via useLexicalScope)
            if scoped_idents.iter().any(|(n, _)| n == name) {
                return None;
            }
            // Check if it's a source export
            export_by_name.get(name).cloned()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_semantic::ScopeId;

    #[test]
    fn test_compute_scoped_idents_empty() {
        let all_idents: Vec<Id> = vec![];
        let all_decl: Vec<IdPlusType> = vec![];
        let (scoped, is_const) = compute_scoped_idents(&all_idents, &all_decl);
        assert!(scoped.is_empty());
        assert!(is_const);
    }

    #[test]
    fn test_compute_scoped_idents_captures_var() {
        let scope_id = ScopeId::new(0);
        let all_idents: Vec<Id> = vec![("foo".to_string(), scope_id)];
        let all_decl: Vec<IdPlusType> = vec![(("foo".to_string(), scope_id), IdentType::Var(false))];
        let (scoped, is_const) = compute_scoped_idents(&all_idents, &all_decl);
        assert_eq!(scoped.len(), 1);
        assert_eq!(scoped[0].0, "foo");
        assert!(!is_const);
    }

    #[test]
    fn test_compute_scoped_idents_captures_const() {
        let scope_id = ScopeId::new(0);
        let all_idents: Vec<Id> = vec![("bar".to_string(), scope_id)];
        let all_decl: Vec<IdPlusType> = vec![(("bar".to_string(), scope_id), IdentType::Var(true))];
        let (scoped, is_const) = compute_scoped_idents(&all_idents, &all_decl);
        assert_eq!(scoped.len(), 1);
        assert_eq!(scoped[0].0, "bar");
        assert!(is_const);
    }

    #[test]
    fn test_build_display_name_simple() {
        use crate::segment::Segment;
        let stack = vec![Segment::Named("Foo".to_string())];
        assert_eq!(build_display_name(&stack), "Foo");
    }

    #[test]
    fn test_build_display_name_nested() {
        use crate::segment::Segment;
        let stack = vec![
            Segment::Named("Foo".to_string()),
            Segment::NamedQrl("onClick".to_string(), 0),
        ];
        assert_eq!(build_display_name(&stack), "Foo_onClick");
    }

    #[test]
    fn test_compute_hash_stable() {
        use std::path::PathBuf;
        let path = PathBuf::from("test.tsx");
        let hash1 = compute_hash(&path, "Foo_component", None);
        let hash2 = compute_hash(&path, "Foo_component", None);
        assert_eq!(hash1, hash2);
    }
}
