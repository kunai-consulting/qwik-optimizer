use crate::component::{Language, SourceInfo};
use crate::prelude::*;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    ScriptFile {
        text: String,
        source_info: SourceInfo,
    },
}

impl Source {
    pub fn from_file<P: AsRef<Path>>(path_buf: P) -> Result<Self> {
        let text = fs::read_to_string(&path_buf)?;
        let source_info = SourceInfo::new(&path_buf)?;
        Ok(Source::ScriptFile { text, source_info })
    }

    pub fn from_source<T: AsRef<str>>(
        text: T,
        language: Language,
        name: Option<String>,
    ) -> Result<Self> {
        let name = name.unwrap_or("script".into());

        let path_str = format!("./{}.{}", name, language.extension());
        let path = Path::new(path_str.as_str());
        let source_info = SourceInfo::new(path)?;
        let text = text.as_ref().to_string();
        Ok(Source::ScriptFile { text, source_info })
    }

    pub fn source_code(&self) -> &str {
        match self {
            Source::ScriptFile { text, .. } => text.as_ref(),
        }
    }

    pub fn source_info(&self) -> &SourceInfo {
        match self {
            Source::ScriptFile { source_info, .. } => source_info,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FILE: &str = "./src/test_input/test_example_1.tsx";

    #[test]
    fn can_load_from_file() {
        let source = Source::from_file(TEST_FILE).unwrap();
        let expected_source = fs::read_to_string(TEST_FILE).unwrap();
        let expected_source_info = SourceInfo::new(TEST_FILE).unwrap();

        assert_eq!(source.source_info(), &expected_source_info);
        assert_eq!(source.source_code(), &expected_source);
    }
}
