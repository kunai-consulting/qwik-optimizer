#![allow(unused)]

use crate::error::Error;
use crate::prelude::*;
use crate::ast_builder_ext::*;
use base64::*;
use oxc::allocator::{Allocator, Box as OxcBox, CloneIn, Vec as OxcVec};
use oxc::ast::ast::*;
use oxc::ast::AstBuilder;
use oxc::codegen::Codegen;
use oxc::span::{SourceType, SPAN};
use std::cell::Cell;
use std::ffi::OsStr;
use std::hash::*;
use std::path::*;


const BUILDER_IO_QWIK: &str = "@builder.io/qwik";

pub enum CommonImport {
    BuilderIoQwik(String),
}

impl CommonImport {
    
    fn gen(self, ast_builder: AstBuilder) -> Statement {
        match self { 
            CommonImport::BuilderIoQwik(name) => ast_builder.qwik_import(name.as_str(), BUILDER_IO_QWIK),
        }
    }
    
}

pub enum CommonExport {
    BuilderIoQwik(String),
}

impl CommonExport {
    
    fn gen(self, ast_builder: AstBuilder) -> Statement {
        match self { 
            CommonExport::BuilderIoQwik(name) => ast_builder.qwik_export(name.as_str(), BUILDER_IO_QWIK),
        }
    }
    
}


/// Renamed from `EmitMode` in V 1.0.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    Prod,
    Lib,
    Dev,
    Test,
}

pub struct QwikComponent<'a> {
    pub id: Id,
    pub source_type: SourceType,
    pub function: ArrowFunctionExpression<'a>,
}

impl<'a> QwikComponent<'a> {
    pub fn new(
        source_info: &SourceInfo,
        segments: &Vec<&str>,
        function: ArrowFunctionExpression<'a>,
        target: &Target,
        scope: &Option<String>,
    ) -> Result<QwikComponent<'a>> {
        let id = Id::new(source_info, segments, target, scope);
        let source_type = source_info.try_into()?;
        Ok(QwikComponent {
            id,
            source_type,
            function,
        })
    }
    
    fn std_import(ast_builder: &AstBuilder)  {
        let imported = ast_builder.module_export_name_identifier_name(SPAN, "qrl");
        let local_name = ast_builder.binding_identifier(SPAN, "qrl");
        let import_specifier = ast_builder.import_specifier(SPAN, imported, local_name, ImportOrExportKind::Value);
        
    }

    pub fn gen(&self, allocator: &Allocator) -> String {
        let name = &self.id.symbol_name;

        let ast_builder = AstBuilder::new(allocator);
        
        Self::std_import(&ast_builder);

        let id = OxcBox::new_in(ast_builder.binding_identifier(SPAN, name), allocator);
        let bind_pat = ast_builder.binding_pattern(
            BindingPatternKind::BindingIdentifier(id),
            None::<OxcBox<'a, TSTypeAnnotation<'a>>>,
            false,
        );
        let mut var_declarator = OxcVec::new_in(allocator);

        let boxed = OxcBox::new_in(self.function.clone_in(allocator), allocator);
        let expr = Expression::ArrowFunctionExpression(boxed);
        var_declarator.push(ast_builder.variable_declarator(
            SPAN,
            VariableDeclarationKind::Const,
            bind_pat,
            Some(expr),
            false,
        ));

        let decl = ast_builder.variable_declaration(
            SPAN,
            VariableDeclarationKind::Const,
            var_declarator,
            false,
        );
        let decl = OxcBox::new_in(decl, allocator);
        let decl = Declaration::VariableDeclaration(decl);
        let export = ast_builder.export_named_declaration(
            SPAN,
            Some(decl),
            OxcVec::new_in(allocator),
            None,
            ImportOrExportKind::Value,
            None::<OxcBox<WithClause>>,
        );
        let export = Statement::ExportNamedDeclaration(OxcBox::new_in(export, allocator));

        let mut body = OxcVec::new_in(allocator);
        body.push(export);
        
        let hw_export = CommonExport::BuilderIoQwik("_hW".into()).gen(ast_builder);
        body.push(hw_export);

        let new_pgm = Program {
            span: SPAN,
            source_type: self.source_type,
            source_text: "",
            comments: OxcVec::new_in(allocator),
            hashbang: None,
            directives: OxcVec::new_in(allocator),
            body,
            scope_id: Cell::new(None),
        };

        let codegen = Codegen::new();
        codegen.build(&new_pgm).code
    }
}

/// Represents a component identifier, including its display name, symbol name, local file name, hash, and optional scope.
///
/// This information is used to uniquely identify a component in the Qwik framework.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id {
    pub display_name: String,
    pub symbol_name: String,
    pub local_file_name: String,
    pub hash: String,
    pub scope: Option<String>,
}

impl Id {
    fn sanitize(input: &str) -> String {
        input
            .chars()
            .fold((String::new(), false), |(mut acc, uscore), c| {
                if c.is_ascii_alphanumeric() {
                    acc.push(c);
                    (acc, false)
                } else if uscore {
                    // Never push consecutive underscores.
                    (acc, true)
                } else {
                    acc.push('_');
                    (acc, true)
                }
            })
            .0
    }

    fn calculate_hash(local_file_name: &str, display_name: &str, scope: &Option<String>) -> String {
        let mut hasher = DefaultHasher::new();
        if let Some(scope) = scope {
            hasher.write(scope.as_bytes());
        }
        hasher.write(local_file_name.as_bytes());
        hasher.write(display_name.as_bytes());
        let hash = hasher.finish();
        engine::general_purpose::URL_SAFE_NO_PAD
            .encode(hash.to_le_bytes())
            .replace(['-', '_'], "0")
    }

    /// Creates a component [Id] from a given [SourceInfo], a `Vec[String]` of segment identifiers that relate back the
    /// components location in the source code, a target (prod, lib, dev, test), and an optional scope.
    ///
    /// The [Id] contains enough information to uniquely identify a component.
    ///
    /// # Segments
    ///
    /// Segments represent an order list of identifiers that uniquely reference a component in the source code.
    ///
    /// ## Example
    ///
    /// ```javascript
    /// export const Counter = component$(() => {
    ///   const store = useStore({ count: 0 });
    ///   return (
    ///     <>
    ///       I am a dynamic component. Qwik will download me only when it is time to re-render me after the
    ///       user clicks on the <code>+1</code> button.
    ///       <br />
    ///       Current count: {store.count}
    ///       <br />
    ///       <button onClick$={() => store.count++}>+1</button>
    ///     </>
    ///   );
    /// });
    /// ```
    /// For this example, the segments that would be provided to [SourceInfo::new] would be: [Counter, component, button, onClick].
    ///
    /// # Target
    ///
    /// The provide [Target] will determine how the [`Id.symbol_name`](field@Id::symbol_name) is generated.
    ///
    /// When [Target::Dev] or [Target::Test] is provided, the symbol name will be generated as `{display_name}_{hash}`.
    ///
    /// ## Examples
    ///
    /// If display_name is `a_b_c` and the hash is `0RVAWYCCxyk`, the symbol name will be `a_b_c_0RVAWYCCxyk`.
    ///
    /// When [Target::Lib] or [Target::Prod] is provided, the symbol name will be generated as `s_{hash}`.
    ///
    /// ## Examples
    ///
    /// If display_name is `a_b_c` and the hash is `0RVAWYCCxyk`, the symbol name will be `s_0RVAWYCCxyk`.
    ///
    ///
    /// # Hash Generation Semantics
    ///
    /// The hash is generated by creating a `DefaultHasher` and writing the following values, converted to bytes, to it:
    /// - The calculated `display_name`
    /// - The [`SourceInfo::rel_path`](field@SourceInfo::rel_path)
    /// - The `scope` (if provided).
    ///
    /// [V 1.0 REF] see `QwikTransform.register_context_name` in `transform.rs.
    pub fn new(
        source_info: &SourceInfo,
        segments: &Vec<&str>,
        target: &Target,
        scope: &Option<String>,
    ) -> Id {
        let local_file_name = source_info.rel_path.to_string_lossy();

        let mut display_name = String::new();

        for segment in segments {
            if display_name.is_empty()
                && segment
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            {
                display_name = format!("_{}", segment);
            } else {
                let prefix: String = if display_name.is_empty() {
                    "".to_string()
                } else {
                    format!("{}_", display_name).to_string()
                };
                display_name = format!("{}{}", prefix, segment);
            }
        }
        display_name = Self::sanitize(&display_name);

        let hash64 = Self::calculate_hash(&local_file_name, &display_name, scope);

        let symbol_name = match target {
            Target::Dev | Target::Test => format!("{}_{}", display_name, hash64),
            Target::Lib | Target::Prod => format!("s_{}", hash64),
        };

        let display_name = format!("{}_{}", &source_info.file_name, display_name);

        Id {
            display_name,
            symbol_name,
            local_file_name: source_info.file_name.clone(),
            hash: hash64,
            scope: scope.clone(),
        }
    }
}

/// Contains information about the source file, including its absolute and relative paths, directory paths.
/// Renamed from `PathData` in V 1.0.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceInfo {
    pub rel_path: PathBuf,
    pub rel_dir: PathBuf,
    pub file_name: String,
}

impl SourceInfo {
    /// Normalizes a path by "squashing" `ParentDir` components (e.g., "..") and ensuring it ends with a slash.
    fn normalize_path(path: &Path) -> PathBuf {
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match &component {
                Component::ParentDir => {
                    if !normalized.pop() {
                        normalized.push(component);
                    }
                }
                _ => {
                    normalized.push(component);
                }
            }
        }
        normalized
    }

    /// Creates a new `SourceInfo` instance from a source file path and a base directory.
    ///
    /// From this information it computes the absolute path, relative path, absolute directory, relative directory,
    /// file stem (file name less the extension), file name, and file extension.
    ///
    /// # Arguments
    /// - src - source file.  e.g. `./app.js`
    pub fn new(src: &str) -> Result<SourceInfo> {
        let path = Path::new(src);
        let rel_dir = path.parent().map(|p| p.to_path_buf()).ok_or_else(|| {
            Error::StringConversion(
                path.to_string_lossy().to_string(),
                "Computing relative directory".to_string(),
            )
        })?;

        let file_name = path.file_name().and_then(OsStr::to_str).ok_or_else(|| {
            Error::StringConversion(
                path.to_string_lossy().to_string(),
                "Computing file name".to_string(),
            )
        })?;

        Ok(SourceInfo {
            rel_path: path.into(),
            rel_dir,
            file_name: file_name.into(),
        })
    }
}

impl TryInto<SourceType> for &SourceInfo {
    type Error = Error;

    fn try_into(self) -> std::result::Result<SourceType, Self::Error> {
        SourceType::from_path(&self.rel_path).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_calculate_hash() {
        let hash0 = Id::calculate_hash("./app.js", "a_b_c", &None);
        let hash1 = Id::calculate_hash("./app.js", "a_b_c", &Some("scope".to_string()));
        assert_eq!(hash0, "0RVAWYCCxyk");
        assert_ne!(hash1, hash0);
    }

    #[test]
    fn test_source_info() {
        let source_info = SourceInfo::new("./app.js").unwrap();
        println!("{:?}", source_info);
        assert_eq!(source_info.rel_path, Path::new("./app.js"));
        assert_eq!(source_info.rel_dir, Path::new("./"));
        assert_eq!(source_info.file_name, "app.js");
    }

    #[test]
    fn creates_a_id() {
        let source_info0 = SourceInfo::new("./app.js").unwrap();
        let id0 = Id::new(
            &source_info0,
            &vec!["a", "b", "c"],
            &Target::Dev,
            &Option::None,
        );
        let hash0 = Id::calculate_hash("./app.js", "a_b_c", &None);

        let expected0 = Id {
            display_name: "app.js_a_b_c".to_string(),
            symbol_name: format!("a_b_c_{}", hash0),
            local_file_name: "app.js".to_string(),
            hash: hash0,
            scope: None,
        };

        let scope1 = Some("scope".to_string());
        let id1 = Id::new(&source_info0, &vec!["1", "b", "c"], &Target::Prod, &scope1);
        // Leading  segments that are digits are prefixed with an additional underscore.
        let hash1 = Id::calculate_hash("./app.js", "_1_b_c", &scope1);
        let expected1 = Id {
            display_name: "app.js__1_b_c".to_string(),
            // When Target is neither "Dev" nor "Test", the symbol name is set to "s_{hash}".
            symbol_name: format!("s_{}", hash1),
            local_file_name: "app.js".to_string(),
            hash: hash1,
            scope: Some("scope".to_string()),
        };

        assert_eq!(id0, expected0);
        assert_eq!(id1, expected1);
    }

    #[test]
    fn escapes_a_name() {
        let name0 = Id::sanitize("a'b-c");
        let name1 = Id::sanitize("A123b_c-~45");
        assert_eq!(name0, "a_b_c");
        assert_eq!(name1, "A123b_c_45");
    }

    #[test]
    fn properly_normalize_path() {
        let path0 = Path::new("/a/b/c");
        let norm0 = SourceInfo::normalize_path(path0);

        let path1 = Path::new("/a/b/../c/"); // Path will be normalized to /a/c/
        let norm1 = SourceInfo::normalize_path(path1);

        let path2 = Path::new("/a/b/c/"); // Path will be normalized to /a/c/
        let norm2 = SourceInfo::normalize_path(path2);

        assert_eq!(norm0, Path::new("/a/b/c/"));
        assert_eq!(norm1, Path::new("/a/c/"));
        assert_eq!(norm2, Path::new("/a/b/c/"));
        assert_eq!(norm0, norm2);
    }
}
