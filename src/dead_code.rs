use oxc_allocator::Box as OxcBox;
use oxc_ast::ast::{ClassElement, FunctionBody, Statement};

pub trait DeadCode {
    fn is_dead_code(&self) -> bool;
}

impl DeadCode for OxcBox<'_, FunctionBody<'_>> {
    fn is_dead_code(&self) -> bool {
        let body_empty = self.is_empty();
        let statements_empty = &self.statements.is_empty();
        let statements_all_dead = self.statements.iter().all(|stmt| stmt.is_dead_code());

        body_empty && *statements_empty && statements_all_dead
    }
}

impl DeadCode for Statement<'_> {
    fn is_dead_code(&self) -> bool {
        match self {
            Statement::TryStatement(s) => s.block.body.is_empty(),
            Statement::FunctionDeclaration(func) => {
                if let Some(body) = &func.body {
                    body.is_dead_code()
                } else {
                    false
                }
            }
            Statement::ClassDeclaration(class) => {
                let elements = &class.body.body;

                for element in elements {
                    match element {
                        ClassElement::StaticBlock(block) => {
                            let empty = block.body.is_empty();
                            let all_dead = block.body.iter().all(|stmt| stmt.is_dead_code());
                            if !empty || !all_dead {
                                return false;
                            }
                        }
                        ClassElement::MethodDefinition(method) => {
                            let body = &method.value;
                            if let Some(body) = &body.body {
                                if !body.is_dead_code() {
                                    return false;
                                }
                            }
                        }
                        ClassElement::PropertyDefinition(prop) => {
                            if prop.value.is_some() {
                                return false;
                            }
                        }
                        ClassElement::AccessorProperty(prop) => {
                            if prop.value.is_some() {
                                return false;
                            }
                        }

                        ClassElement::TSIndexSignature(_) => {}
                    }
                }
                true
            }
            _ => false,
        }
    }
}
