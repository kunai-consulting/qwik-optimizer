use std::collections::HashSet;

use base64::{engine, Engine};
use std::hash::{DefaultHasher, Hasher};

use crate::collector::Id;

use super::generator::{IdentType, IdPlusType};

pub fn compute_scoped_idents(all_idents: &[Id], all_decl: &[IdPlusType]) -> (Vec<Id>, bool) {
    let mut set: HashSet<Id> = HashSet::new();
    let mut is_const = true;

    for ident in all_idents {
        if let Some(item) = all_decl.iter().find(|item| item.0 .0 == ident.0) {
            set.insert(item.0.clone());
            if !matches!(item.1, IdentType::Var(true)) {
                is_const = false;
            }
        }
    }

    let mut output: Vec<Id> = set.into_iter().collect();
    output.sort();
    (output, is_const)
}

pub(crate) fn build_display_name(segment_stack: &[crate::segment::Segment]) -> String {
    use crate::segment::Segment;

    let mut display_name = String::new();

    for segment in segment_stack {
        let segment_str: String = match segment {
            Segment::Named(name) => name.clone(),
            Segment::NamedQrl(name, 0) => name.clone(),
            Segment::NamedQrl(name, index) => format!("{}_{}", name, index),
            Segment::IndexQrl(0) => continue,
            Segment::IndexQrl(index) => index.to_string(),
        };

        if segment_str.is_empty() {
            continue;
        }

        if display_name.is_empty() {
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

pub(crate) fn collect_imported_names(imports: &[crate::component::Import]) -> HashSet<String> {
    use crate::component::ImportId;

    imports
        .iter()
        .flat_map(|import| import.names.iter())
        .filter_map(|id| match id {
            ImportId::Named(name) => Some(name.clone()),
            ImportId::Default(name) => Some(name.clone()),
            ImportId::NamedWithAlias(_, local) => Some(local.clone()),
            ImportId::Namespace(_) => None,
        })
        .collect()
}

pub(crate) fn filter_imported_from_scoped(
    scoped_idents: Vec<Id>,
    imported_names: &HashSet<String>,
) -> Vec<Id> {
    scoped_idents
        .into_iter()
        .filter(|(name, _)| !imported_names.contains(name))
        .collect()
}

pub(crate) fn collect_referenced_exports(
    descendent_idents: &[Id],
    imported_names: &HashSet<String>,
    scoped_idents: &[Id],
    export_by_name: &std::collections::HashMap<String, crate::collector::ExportInfo>,
) -> Vec<crate::collector::ExportInfo> {
    descendent_idents
        .iter()
        .filter_map(|(name, _)| {
            if imported_names.contains(name) {
                return None;
            }
            if scoped_idents.iter().any(|(n, _)| n == name) {
                return None;
            }
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
