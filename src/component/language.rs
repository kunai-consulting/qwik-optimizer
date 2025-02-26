use std::path::Path;
use oxc_span::SourceType;
use crate::error::Error;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Language {
    Javascript,
    Typescript,
}

impl<'a> TryFrom<&'a Path> for Language {
    type Error = Error;

    fn try_from(path: &'a Path) -> Result<Language> {
        let source_type = SourceType::from_path(path)?;
        source_type.try_into()
    }
}

impl TryFrom<SourceType> for Language {
    type Error = Error;
    
    fn try_from(source_type: SourceType) -> Result<Language> {
      if source_type.is_javascript() || source_type.is_jsx() {  
          Ok(Language::Javascript)
      } else if source_type.is_typescript() || source_type.is_typescript_definition() {
          Ok(Language::Typescript)
      } else {
          Err(Error::UnsupportedLanguage(
              format!("{:?}", source_type),
          ))
      }
    }
}

impl Into<SourceType> for Language {
    fn into(self) -> SourceType {
        match self {
            Language::Javascript => SourceType::jsx(),
            Language::Typescript => SourceType::tsx(),
        }
    }
}
