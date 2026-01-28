use oxc_ast::ast::Statement;
use oxc_semantic::SymbolId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IllegalCodeType {
    Class(SymbolId, Option<String>),
    Function(SymbolId, Option<String>),
}

impl IllegalCodeType {
    pub(crate) fn symbol_id(&self) -> SymbolId {
        match self {
            IllegalCodeType::Class(id, _) => *id,
            IllegalCodeType::Function(id, _) => *id,
        }
    }

    pub(crate) fn expression_type(&self) -> &str {
        match self {
            IllegalCodeType::Class(_, _) => "class",
            IllegalCodeType::Function(_, _) => "function",
        }
    }

    pub(crate) fn identifier(&self) -> String {
        match self {
            IllegalCodeType::Class(_, name) => name
                .as_ref()
                .map_or("<ANONYMOUS>".to_string(), |s| s.clone()),
            IllegalCodeType::Function(_, name) => name
                .as_ref()
                .map_or("<ANONYMOUS>".to_string(), |s| s.clone()),
        }
    }
}

pub(crate) trait IllegalCode {
    fn is_illegal_code_in_qrl(&self) -> Option<IllegalCodeType>;
}

impl IllegalCode for Statement<'_> {
    fn is_illegal_code_in_qrl(&self) -> Option<IllegalCodeType> {
        match self {
            Statement::FunctionDeclaration(fd) => {
                let bid = fd.id.clone();
                bid.and_then(|id| id.symbol_id.get()).map(|symbol_id| {
                    IllegalCodeType::Function(symbol_id, fd.name().map(String::from))
                })
            }
            Statement::ClassDeclaration(cd) => {
                let bid = cd.id.clone();
                bid.and_then(|bid| bid.symbol_id.get())
                    .map(|id| IllegalCodeType::Class(id, cd.name().map(String::from)))
            }
            _s => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
    use oxc_span::SourceType;

    fn parse_statement<'a>(source: &'a str, allocator: &'a Allocator) -> Statement<'a> {
        let source_type = SourceType::default();
        let ret = Parser::new(&allocator, source, source_type).parse();
        let program = ret.program;

        let SemanticBuilderReturn {
            semantic: _,
            errors: _,
        } = SemanticBuilder::new()
            .with_check_syntax_error(true) // Enable extra syntax error checking
            .with_cfg(true) // Build a Control Flow Graph
            .build(&program);

        program
            .body
            .into_iter()
            .next()
            .expect("Should have at least one statement")
    }

    #[test]
    fn test_function_declaration_is_illegal() {
        let allocator = Allocator::default();
        let stmt = parse_statement("function foo() {}", &allocator);
        let result = stmt.is_illegal_code_in_qrl();
        if let Some(IllegalCodeType::Function(_, name)) = result {
            assert_eq!(name, Some("foo".to_string()));
        } else {
            panic!("Expected function declaration to be illegal code");
        }
    }

    #[test]
    fn test_class_declaration_is_illegal() {
        let allocator = Allocator::default();
        let stmt = parse_statement("class Bar {}", &allocator);
        if let Some(IllegalCodeType::Class(_, name)) = stmt.is_illegal_code_in_qrl() {
            assert_eq!(name, Some("Bar".to_string()));
        } else {
            panic!("Expected class declaration to be illegal code");
        }
    }

    #[test]
    fn test_non_illegal_statement() {
        let allocator = Allocator::default();
        let stmt = parse_statement("let x = 1;", &allocator);
        assert_eq!(
            stmt.is_illegal_code_in_qrl(),
            None,
            "Variable declaration should not be illegal code"
        );
    }
}
