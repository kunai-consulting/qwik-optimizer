//! # SSR & Build Mode Const Replacement
//!
//! This module implements const replacement for SSR/build mode identifiers,
//! satisfying the following requirements:
//!
//! - **SSR-01**: isServer const replaced correctly based on build target
//! - **SSR-02**: isDev const replaced correctly based on build mode
//! - **SSR-03**: Server-only code marked for elimination (if(false) pattern)
//! - **SSR-04**: Client-only code marked for elimination (if(false) pattern)
//! - **SSR-05**: Mode-specific transformations apply correctly
//!
//! ## How It Works
//!
//! 1. Imports are collected to track which local identifiers map to isServer/isBrowser/isDev
//! 2. ConstReplacerVisitor replaces those identifiers with boolean literals
//! 3. Downstream bundler (Vite/Rollup) performs dead code elimination on if(true)/if(false)
//!
//! ## Configuration
//!
//! - `TransformOptions.is_server`: true for server build, false for client build
//! - `TransformOptions.target`: Dev/Prod/Lib/Test - affects isDev value
//! - Const replacement is skipped in Test mode (matching SWC behavior)
//!
//! ## Example Transformation
//!
//! ```text
//! // Input (server build, is_server=true)
//! import { isServer } from '@qwik.dev/core/build';
//! if (isServer) { serverOnlyCode(); }
//!
//! // Output
//! if (true) { serverOnlyCode(); }
//! // Bundler then eliminates the unreachable branch
//! ```
//!
//! ## Supported Import Sources
//!
//! - `@qwik.dev/core/build` (primary source for build constants)
//! - `@qwik.dev/core` (also supports these exports)

use crate::component::{IS_BROWSER, IS_DEV, IS_SERVER, QWIK_CORE_BUILD, QWIK_CORE_SOURCE};
use crate::transform::ImportTracker;
use oxc_allocator::{Allocator, Box as OxcBox};
use oxc_ast::ast::{self, BooleanLiteral, Expression, Program, Statement};
use oxc_span::SPAN;

/// Which constant variable is being replaced
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstVariable {
    IsServer,
    IsBrowser,
    IsDev,
    None,
}

/// Visitor that replaces isServer/isBrowser/isDev identifiers with boolean literals.
///
/// Tracks which local identifiers map to the build constants (handling aliased imports)
/// and replaces them with the appropriate boolean value.
pub struct ConstReplacerVisitor<'a> {
    /// Reference to the allocator for creating AST nodes
    allocator: &'a Allocator,
    /// Whether this is a server build
    pub is_server: bool,
    /// Whether this is a development build
    pub is_dev: bool,
    /// Local identifier for isServer from @qwik.dev/core/build
    is_server_ident: Option<String>,
    /// Local identifier for isBrowser from @qwik.dev/core/build
    is_browser_ident: Option<String>,
    /// Local identifier for isDev from @qwik.dev/core/build
    is_dev_ident: Option<String>,
    /// Local identifier for isServer from @qwik.dev/core
    is_core_server_ident: Option<String>,
    /// Local identifier for isBrowser from @qwik.dev/core
    is_core_browser_ident: Option<String>,
    /// Local identifier for isDev from @qwik.dev/core
    is_core_dev_ident: Option<String>,
}

impl<'a> ConstReplacerVisitor<'a> {
    /// Create a new ConstReplacerVisitor.
    ///
    /// # Arguments
    /// * `allocator` - The OXC allocator for creating AST nodes
    /// * `is_server` - Whether this is a server build (true) or client build (false)
    /// * `is_dev` - Whether this is a development build
    /// * `import_tracker` - Tracks imported identifiers for finding aliased imports
    pub fn new(
        allocator: &'a Allocator,
        is_server: bool,
        is_dev: bool,
        import_tracker: &ImportTracker,
    ) -> Self {
        let is_server_ident = import_tracker
            .get_imported_local(IS_SERVER, QWIK_CORE_BUILD)
            .cloned();
        let is_browser_ident = import_tracker
            .get_imported_local(IS_BROWSER, QWIK_CORE_BUILD)
            .cloned();
        let is_dev_ident = import_tracker
            .get_imported_local(IS_DEV, QWIK_CORE_BUILD)
            .cloned();
        let is_core_server_ident = import_tracker
            .get_imported_local(IS_SERVER, QWIK_CORE_SOURCE)
            .cloned();
        let is_core_browser_ident = import_tracker
            .get_imported_local(IS_BROWSER, QWIK_CORE_SOURCE)
            .cloned();
        let is_core_dev_ident = import_tracker
            .get_imported_local(IS_DEV, QWIK_CORE_SOURCE)
            .cloned();

        Self {
            allocator,
            is_server,
            is_dev,
            is_server_ident,
            is_browser_ident,
            is_dev_ident,
            is_core_server_ident,
            is_core_browser_ident,
            is_core_dev_ident,
        }
    }

    fn match_ident(&self, name: &str) -> ConstVariable {
        if self.is_server_ident.as_deref() == Some(name) {
            return ConstVariable::IsServer;
        }
        if self.is_browser_ident.as_deref() == Some(name) {
            return ConstVariable::IsBrowser;
        }
        if self.is_dev_ident.as_deref() == Some(name) {
            return ConstVariable::IsDev;
        }
        if self.is_core_server_ident.as_deref() == Some(name) {
            return ConstVariable::IsServer;
        }
        if self.is_core_browser_ident.as_deref() == Some(name) {
            return ConstVariable::IsBrowser;
        }
        if self.is_core_dev_ident.as_deref() == Some(name) {
            return ConstVariable::IsDev;
        }
        ConstVariable::None
    }

    /// Create a boolean literal expression with the given value
    fn make_bool_expr(&self, value: bool) -> Expression<'a> {
        Expression::BooleanLiteral(OxcBox::new_in(
            BooleanLiteral { span: SPAN, value },
            self.allocator,
        ))
    }

    /// Visit and transform a program in place
    pub fn visit_program(&mut self, program: &mut Program<'a>) {
        for stmt in program.body.iter_mut() {
            self.visit_statement(stmt);
        }
    }

    /// Visit a statement and transform any expressions within
    fn visit_statement(&mut self, stmt: &mut Statement<'a>) {
        match stmt {
            Statement::ExpressionStatement(expr_stmt) => {
                self.visit_expression(&mut expr_stmt.expression);
            }
            Statement::VariableDeclaration(decl) => {
                self.visit_variable_declaration(decl);
            }
            Statement::IfStatement(if_stmt) => {
                self.visit_expression(&mut if_stmt.test);
                self.visit_statement(&mut if_stmt.consequent);
                if let Some(alt) = &mut if_stmt.alternate {
                    self.visit_statement(alt);
                }
            }
            Statement::WhileStatement(while_stmt) => {
                self.visit_expression(&mut while_stmt.test);
                self.visit_statement(&mut while_stmt.body);
            }
            Statement::DoWhileStatement(do_while) => {
                self.visit_statement(&mut do_while.body);
                self.visit_expression(&mut do_while.test);
            }
            Statement::ForStatement(for_stmt) => {
                if let Some(init) = &mut for_stmt.init {
                    match init {
                        ast::ForStatementInit::VariableDeclaration(decl) => {
                            self.visit_variable_declaration(decl);
                        }
                        _ => {}
                    }
                }
                if let Some(test) = &mut for_stmt.test {
                    self.visit_expression(test);
                }
                if let Some(update) = &mut for_stmt.update {
                    self.visit_expression(update);
                }
                self.visit_statement(&mut for_stmt.body);
            }
            Statement::BlockStatement(block) => {
                for s in block.body.iter_mut() {
                    self.visit_statement(s);
                }
            }
            Statement::ReturnStatement(ret) => {
                if let Some(arg) = &mut ret.argument {
                    self.visit_expression(arg);
                }
            }
            Statement::FunctionDeclaration(func) => {
                if let Some(body) = &mut func.body {
                    for s in body.statements.iter_mut() {
                        self.visit_statement(s);
                    }
                }
            }
            Statement::ExportNamedDeclaration(export) => {
                self.visit_export_named_declaration(export);
            }
            Statement::ExportDefaultDeclaration(export) => {
                self.visit_export_default_declaration(export);
            }
            _ => {}
        }
    }

    /// Visit a variable declaration
    fn visit_variable_declaration(&mut self, decl: &mut ast::VariableDeclaration<'a>) {
        for declarator in decl.declarations.iter_mut() {
            if let Some(init) = &mut declarator.init {
                self.visit_expression(init);
            }
        }
    }

    /// Visit an export named declaration: `export const foo = isServer;`
    fn visit_export_named_declaration(&mut self, export: &mut ast::ExportNamedDeclaration<'a>) {
        if let Some(decl) = &mut export.declaration {
            match decl {
                ast::Declaration::VariableDeclaration(var_decl) => {
                    self.visit_variable_declaration(var_decl);
                }
                ast::Declaration::FunctionDeclaration(fn_decl) => {
                    if let Some(body) = &mut fn_decl.body {
                        for s in body.statements.iter_mut() {
                            self.visit_statement(s);
                        }
                    }
                }
                ast::Declaration::ClassDeclaration(class_decl) => {
                    // Visit class body for any expressions
                    for element in class_decl.body.body.iter_mut() {
                        match element {
                            ast::ClassElement::PropertyDefinition(prop) => {
                                if let Some(value) = &mut prop.value {
                                    self.visit_expression(value);
                                }
                            }
                            ast::ClassElement::MethodDefinition(method) => {
                                if let Some(body) = &mut method.value.body {
                                    for s in body.statements.iter_mut() {
                                        self.visit_statement(s);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Visit an export default declaration: `export default isServer;`
    fn visit_export_default_declaration(&mut self, export: &mut ast::ExportDefaultDeclaration<'a>) {
        match &mut export.declaration {
            ast::ExportDefaultDeclarationKind::FunctionDeclaration(fn_decl) => {
                if let Some(body) = &mut fn_decl.body {
                    for s in body.statements.iter_mut() {
                        self.visit_statement(s);
                    }
                }
            }
            ast::ExportDefaultDeclarationKind::ClassDeclaration(class_decl) => {
                for element in class_decl.body.body.iter_mut() {
                    match element {
                        ast::ClassElement::PropertyDefinition(prop) => {
                            if let Some(value) = &mut prop.value {
                                self.visit_expression(value);
                            }
                        }
                        ast::ClassElement::MethodDefinition(method) => {
                            if let Some(body) = &mut method.value.body {
                                for s in body.statements.iter_mut() {
                                    self.visit_statement(s);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                // For expressions: `export default isServer;`
                if let Some(expr) = export.declaration.as_expression_mut() {
                    self.visit_expression(expr);
                }
            }
        }
    }

    /// Visit an expression and replace const identifiers with boolean literals
    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        // Check if this expression is an identifier that should be replaced
        let const_var = match expr {
            Expression::Identifier(ident) => self.match_ident(&ident.name),
            _ => ConstVariable::None,
        };

        match const_var {
            ConstVariable::IsServer => {
                *expr = self.make_bool_expr(self.is_server);
            }
            ConstVariable::IsBrowser => {
                *expr = self.make_bool_expr(!self.is_server);
            }
            ConstVariable::IsDev => {
                *expr = self.make_bool_expr(self.is_dev);
            }
            ConstVariable::None => {
                self.visit_expression_children(expr);
            }
        }
    }

    /// Visit children of an expression
    fn visit_expression_children(&mut self, expr: &mut Expression<'a>) {
        match expr {
            Expression::ArrayExpression(arr) => {
                for elem in arr.elements.iter_mut() {
                    match elem {
                        ast::ArrayExpressionElement::SpreadElement(spread) => {
                            self.visit_expression(&mut spread.argument);
                        }
                        ast::ArrayExpressionElement::Elision(_) => {}
                        _ => {
                            if let Some(expr) = elem.as_expression_mut() {
                                self.visit_expression(expr);
                            }
                        }
                    }
                }
            }
            Expression::ObjectExpression(obj) => {
                for prop in obj.properties.iter_mut() {
                    match prop {
                        ast::ObjectPropertyKind::ObjectProperty(p) => {
                            self.visit_expression(&mut p.value);
                        }
                        ast::ObjectPropertyKind::SpreadProperty(spread) => {
                            self.visit_expression(&mut spread.argument);
                        }
                    }
                }
            }
            Expression::CallExpression(call) => {
                if let ast::Expression::Identifier(_) = &call.callee {
                } else {
                    self.visit_expression(&mut call.callee);
                }
                for arg in call.arguments.iter_mut() {
                    match arg {
                        ast::Argument::SpreadElement(spread) => {
                            self.visit_expression(&mut spread.argument);
                        }
                        _ => {
                            if let Some(expr) = arg.as_expression_mut() {
                                self.visit_expression(expr);
                            }
                        }
                    }
                }
            }
            Expression::BinaryExpression(bin) => {
                self.visit_expression(&mut bin.left);
                self.visit_expression(&mut bin.right);
            }
            Expression::LogicalExpression(log) => {
                self.visit_expression(&mut log.left);
                self.visit_expression(&mut log.right);
            }
            Expression::UnaryExpression(unary) => {
                self.visit_expression(&mut unary.argument);
            }
            Expression::ConditionalExpression(cond) => {
                self.visit_expression(&mut cond.test);
                self.visit_expression(&mut cond.consequent);
                self.visit_expression(&mut cond.alternate);
            }
            Expression::AssignmentExpression(assign) => {
                self.visit_expression(&mut assign.right);
            }
            Expression::SequenceExpression(seq) => {
                for expr in seq.expressions.iter_mut() {
                    self.visit_expression(expr);
                }
            }
            Expression::StaticMemberExpression(static_member) => {
                self.visit_expression(&mut static_member.object);
            }
            Expression::ComputedMemberExpression(computed) => {
                self.visit_expression(&mut computed.object);
                self.visit_expression(&mut computed.expression);
            }
            Expression::PrivateFieldExpression(private) => {
                self.visit_expression(&mut private.object);
            }
            Expression::ArrowFunctionExpression(arrow) => {
                for stmt in arrow.body.statements.iter_mut() {
                    self.visit_statement(stmt);
                }
            }
            Expression::FunctionExpression(func) => {
                if let Some(body) = &mut func.body {
                    for stmt in body.statements.iter_mut() {
                        self.visit_statement(stmt);
                    }
                }
            }
            Expression::ParenthesizedExpression(paren) => {
                self.visit_expression(&mut paren.expression);
            }
            Expression::TemplateLiteral(template) => {
                for expr in template.expressions.iter_mut() {
                    self.visit_expression(expr);
                }
            }
            Expression::TaggedTemplateExpression(tagged) => {
                self.visit_expression(&mut tagged.tag);
                for expr in tagged.quasi.expressions.iter_mut() {
                    self.visit_expression(expr);
                }
            }
            Expression::NewExpression(new_expr) => {
                self.visit_expression(&mut new_expr.callee);
                for arg in new_expr.arguments.iter_mut() {
                    match arg {
                        ast::Argument::SpreadElement(spread) => {
                            self.visit_expression(&mut spread.argument);
                        }
                        _ => {
                            if let Some(expr) = arg.as_expression_mut() {
                                self.visit_expression(expr);
                            }
                        }
                    }
                }
            }
            Expression::AwaitExpression(await_expr) => {
                self.visit_expression(&mut await_expr.argument);
            }
            Expression::YieldExpression(yield_expr) => {
                if let Some(arg) = &mut yield_expr.argument {
                    self.visit_expression(arg);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform::ImportTracker;
    use oxc_allocator::Allocator;
    use oxc_ast::ast::ImportDeclarationSpecifier;
    use oxc_codegen::Codegen;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    fn replace_consts(code: &str, is_server: bool, is_dev: bool) -> String {
        let allocator = Allocator::default();
        let source_type = SourceType::tsx();
        let parse_result = Parser::new(&allocator, code, source_type).parse();
        assert!(
            parse_result.errors.is_empty(),
            "Parse errors: {:?}",
            parse_result.errors
        );

        let mut program = parse_result.program;

        // Build import tracker from the parsed program
        let mut import_tracker = ImportTracker::new();
        for stmt in &program.body {
            if let Statement::ImportDeclaration(import) = stmt {
                let source = import.source.value.to_string();
                if let Some(specifiers) = &import.specifiers {
                    for specifier in specifiers {
                        if let ImportDeclarationSpecifier::ImportSpecifier(spec) = specifier {
                            let imported = spec.imported.name().to_string();
                            let local = spec.local.name.to_string();
                            import_tracker.add_import(&source, &imported, &local);
                        }
                    }
                }
            }
        }

        let mut visitor = ConstReplacerVisitor::new(&allocator, is_server, is_dev, &import_tracker);
        visitor.visit_program(&mut program);

        Codegen::default().build(&program).code
    }

    #[test]
    fn test_is_server_replacement_true() {
        let code = r#"
import { isServer } from '@qwik.dev/core/build';
if (isServer) { console.log('server'); }
"#;
        let output = replace_consts(code, true, false);
        assert!(
            output.contains("if (true)"),
            "Expected 'true', got: {}",
            output
        );
    }

    #[test]
    fn test_is_server_replacement_false() {
        let code = r#"
import { isServer } from '@qwik.dev/core/build';
if (isServer) { console.log('server'); }
"#;
        let output = replace_consts(code, false, false);
        assert!(
            output.contains("if (false)"),
            "Expected 'false', got: {}",
            output
        );
    }

    #[test]
    fn test_is_browser_replacement() {
        let code = r#"
import { isBrowser } from '@qwik.dev/core/build';
if (isBrowser) { console.log('browser'); }
"#;
        let output = replace_consts(code, true, false);
        assert!(
            output.contains("if (false)"),
            "Expected 'false' (isBrowser when server), got: {}",
            output
        );

        let output = replace_consts(code, false, false);
        assert!(
            output.contains("if (true)"),
            "Expected 'true' (isBrowser when client), got: {}",
            output
        );
    }

    #[test]
    fn test_is_dev_replacement() {
        let code = r#"
import { isDev } from '@qwik.dev/core/build';
if (isDev) { console.log('dev'); }
"#;
        let output = replace_consts(code, true, true);
        assert!(
            output.contains("if (true)"),
            "Expected 'true' (isDev=true), got: {}",
            output
        );

        let output = replace_consts(code, true, false);
        assert!(
            output.contains("if (false)"),
            "Expected 'false' (isDev=false), got: {}",
            output
        );
    }

    #[test]
    fn test_aliased_import() {
        let code = r#"
import { isServer as s } from '@qwik.dev/core/build';
if (s) { console.log('server'); }
"#;
        let output = replace_consts(code, true, false);
        assert!(
            output.contains("if (true)"),
            "Expected aliased 'isServer as s' to be replaced, got: {}",
            output
        );
    }

    #[test]
    fn test_qwik_core_source() {
        // isServer can also be imported from @qwik.dev/core
        let code = r#"
import { isServer } from '@qwik.dev/core';
if (isServer) { console.log('server'); }
"#;
        let output = replace_consts(code, true, false);
        assert!(
            output.contains("if (true)"),
            "Expected @qwik.dev/core import to work, got: {}",
            output
        );
    }

    #[test]
    fn test_multiple_consts() {
        let code = r#"
import { isServer, isBrowser, isDev } from '@qwik.dev/core/build';
const a = isServer;
const b = isBrowser;
const c = isDev;
"#;
        let output = replace_consts(code, true, true);
        assert!(
            output.contains("const a = true"),
            "isServer should be true, got: {}",
            output
        );
        assert!(
            output.contains("const b = false"),
            "isBrowser should be false, got: {}",
            output
        );
        assert!(
            output.contains("const c = true"),
            "isDev should be true, got: {}",
            output
        );
    }

    #[test]
    fn test_non_imported_identifier_unchanged() {
        // If isServer is not imported, it shouldn't be replaced
        let code = r#"
const isServer = false;
if (isServer) { console.log('local var'); }
"#;
        let output = replace_consts(code, true, false);
        assert!(
            output.contains("isServer"),
            "Local isServer should not be replaced"
        );
    }

    #[test]
    fn test_nested_expression() {
        let code = r#"
import { isServer } from '@qwik.dev/core/build';
const x = isServer ? 'server' : 'client';
"#;
        let output = replace_consts(code, true, false);
        assert!(
            output.contains("true ?"),
            "Ternary condition should have true, got: {}",
            output
        );
    }

    #[test]
    fn test_logical_expression() {
        let code = r#"
import { isServer, isDev } from '@qwik.dev/core/build';
const x = isServer && isDev;
"#;
        let output = replace_consts(code, true, true);
        assert!(
            output.contains("true && true"),
            "Both should be true, got: {}",
            output
        );
    }

    #[test]
    fn test_both_import_sources() {
        // Test that both @qwik.dev/core and @qwik.dev/core/build work together
        let code = r#"
import { isServer } from '@qwik.dev/core';
import { isDev } from '@qwik.dev/core/build';
const a = isServer;
const b = isDev;
"#;
        let output = replace_consts(code, true, true);
        assert!(
            output.contains("const a = true"),
            "isServer from core should work, got: {}",
            output
        );
        assert!(
            output.contains("const b = true"),
            "isDev from core/build should work, got: {}",
            output
        );
    }

    #[test]
    fn test_no_replacement_without_import() {
        // Variables with same names but not imported should not be replaced
        let code = r#"
function test() {
    const isServer = computeIsServer();
    const isBrowser = !isServer;
    return { isServer, isBrowser };
}
"#;
        let output = replace_consts(code, true, false);
        // These are local variables, not imports - should remain unchanged
        assert!(
            output.contains("computeIsServer()"),
            "Local variables should not be touched, got: {}",
            output
        );
    }
}
