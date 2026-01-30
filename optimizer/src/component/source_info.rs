use crate::component::Language;
use crate::error::*;
use crate::prelude::*;
use oxc_span::SourceType;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Source file information including paths and language.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceInfo {
    pub rel_path: PathBuf,
    pub rel_dir: PathBuf,
    pub file_name: String,
    pub language: Language,
}

impl SourceInfo {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<SourceInfo> {
        let path = path.as_ref();
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

        let language = Language::try_from(path)?;

        Ok(SourceInfo {
            rel_path: path.into(),
            rel_dir,
            file_name: file_name.into(),
            language,
        })
    }

    pub fn rel_import_path(&self) -> PathBuf {
        match self.language {
            Language::Javascript => self.rel_path.clone(),
            Language::Typescript => self.rel_path.clone().with_extension(""),
        }
    }
}

impl TryInto<SourceType> for &SourceInfo {
    type Error = Error;

    fn try_into(self) -> std::result::Result<SourceType, Self::Error> {
        SourceType::from_path(&self.rel_path).map_err(|e| e.into())
    }
}

impl TryInto<SourceType> for SourceInfo {
    type Error = Error;

    fn try_into(self) -> std::result::Result<SourceType, Self::Error> {
        SourceType::from_path(&self.rel_path).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Component;

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
