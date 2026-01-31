#[cfg(test)]
mod tests {
    use crate::collector::Id;
    use crate::transform::*;
    use oxc_semantic::ScopeId;

    // ==================== Internal API Tests ====================
    // These tests verify internal helper functions that are not directly
    // exercised by the spec parity test suite. They test implementation
    // details and edge cases of helper functions.

    // ==================== compute_scoped_idents Tests (5 tests) ====================

    #[test]
    fn test_compute_scoped_idents_basic() {
        let idents: Vec<Id> = vec![
            ("a".to_string(), ScopeId::new(0)),
            ("b".to_string(), ScopeId::new(0)),
            ("c".to_string(), ScopeId::new(0)),
        ];
        let decls: Vec<IdPlusType> = vec![
            (("a".to_string(), ScopeId::new(0)), IdentType::Var(true)),
            (("b".to_string(), ScopeId::new(0)), IdentType::Var(false)),
        ];

        let (scoped, is_const) = compute_scoped_idents(&idents, &decls);

        assert_eq!(scoped.len(), 2);
        assert!(scoped.contains(&("a".to_string(), ScopeId::new(0))));
        assert!(scoped.contains(&("b".to_string(), ScopeId::new(0))));
        assert!(!is_const);
    }

    #[test]
    fn test_compute_scoped_idents_all_const() {
        let idents: Vec<Id> = vec![
            ("x".to_string(), ScopeId::new(0)),
            ("y".to_string(), ScopeId::new(0)),
        ];
        let decls: Vec<IdPlusType> = vec![
            (("x".to_string(), ScopeId::new(0)), IdentType::Var(true)),
            (("y".to_string(), ScopeId::new(0)), IdentType::Var(true)),
        ];

        let (scoped, is_const) = compute_scoped_idents(&idents, &decls);

        assert_eq!(scoped.len(), 2);
        assert!(is_const);
    }

    #[test]
    fn test_compute_scoped_idents_fn_class_not_const() {
        let idents: Vec<Id> = vec![
            ("myFn".to_string(), ScopeId::new(0)),
            ("MyClass".to_string(), ScopeId::new(0)),
        ];
        let decls: Vec<IdPlusType> = vec![
            (("myFn".to_string(), ScopeId::new(0)), IdentType::Fn),
            (("MyClass".to_string(), ScopeId::new(0)), IdentType::Class),
        ];

        let (scoped, is_const) = compute_scoped_idents(&idents, &decls);

        assert_eq!(scoped.len(), 2);
        assert!(!is_const);
    }

    #[test]
    fn test_compute_scoped_idents_sorted_output() {
        let idents: Vec<Id> = vec![
            ("z".to_string(), ScopeId::new(0)),
            ("a".to_string(), ScopeId::new(0)),
            ("m".to_string(), ScopeId::new(0)),
        ];
        let decls: Vec<IdPlusType> = vec![
            (("z".to_string(), ScopeId::new(0)), IdentType::Var(true)),
            (("a".to_string(), ScopeId::new(0)), IdentType::Var(true)),
            (("m".to_string(), ScopeId::new(0)), IdentType::Var(true)),
        ];

        let (scoped, _) = compute_scoped_idents(&idents, &decls);

        assert_eq!(
            scoped,
            vec![
                ("a".to_string(), ScopeId::new(0)),
                ("m".to_string(), ScopeId::new(0)),
                ("z".to_string(), ScopeId::new(0)),
            ]
        );
    }

    #[test]
    fn test_compute_scoped_idents_empty() {
        let idents: Vec<Id> = vec![("a".to_string(), ScopeId::new(0))];
        let decls: Vec<IdPlusType> = vec![];

        let (scoped, is_const) = compute_scoped_idents(&idents, &decls);

        assert!(scoped.is_empty());
        assert!(is_const);
    }

    // ==================== jsx_event_to_html_attribute Tests (4 tests) ====================

    #[test]
    fn test_jsx_event_to_html_attribute_basic() {
        assert_eq!(jsx_event_to_html_attribute("onClick$"), Some("on:click".to_string()));
        assert_eq!(jsx_event_to_html_attribute("onInput$"), Some("on:input".to_string()));
        assert_eq!(jsx_event_to_html_attribute("onDblClick$"), Some("on:dblclick".to_string()));
        assert_eq!(jsx_event_to_html_attribute("onKeyDown$"), Some("on:keydown".to_string()));
        assert_eq!(jsx_event_to_html_attribute("onMouseOver$"), Some("on:mouseover".to_string()));
        assert_eq!(jsx_event_to_html_attribute("onBlur$"), Some("on:blur".to_string()));
    }

    #[test]
    fn test_jsx_event_to_html_attribute_document_window() {
        assert_eq!(jsx_event_to_html_attribute("document:onFocus$"), Some("on-document:focus".to_string()));
        assert_eq!(jsx_event_to_html_attribute("document:onClick$"), Some("on-document:click".to_string()));
        assert_eq!(jsx_event_to_html_attribute("window:onClick$"), Some("on-window:click".to_string()));
        assert_eq!(jsx_event_to_html_attribute("window:onScroll$"), Some("on-window:scroll".to_string()));
    }

    #[test]
    fn test_jsx_event_to_html_attribute_case_preserving() {
        assert_eq!(jsx_event_to_html_attribute("on-cLick$"), Some("on:c-lick".to_string()));
        assert_eq!(jsx_event_to_html_attribute("on-anotherCustom$"), Some("on:another-custom".to_string()));
    }

    #[test]
    fn test_jsx_event_to_html_attribute_not_event() {
        assert_eq!(jsx_event_to_html_attribute("onClick"), None);
        assert_eq!(jsx_event_to_html_attribute("custom$"), None);
        assert_eq!(jsx_event_to_html_attribute("$"), None);
        assert_eq!(jsx_event_to_html_attribute(""), None);
    }

    // ==================== get_event_scope_data Test (1 test) ====================

    #[test]
    fn test_get_event_scope_data() {
        assert_eq!(get_event_scope_data_from_jsx_event("onClick$"), ("on:", 2));
        assert_eq!(get_event_scope_data_from_jsx_event("onInput$"), ("on:", 2));
        assert_eq!(get_event_scope_data_from_jsx_event("document:onFocus$"), ("on-document:", 11));
        assert_eq!(get_event_scope_data_from_jsx_event("window:onClick$"), ("on-window:", 9));
        assert_eq!(get_event_scope_data_from_jsx_event("custom$"), ("", usize::MAX));
    }

    // ==================== should_wrap_in_fn_signal Tests (4 tests) ====================

    #[test]
    fn test_should_wrap_in_fn_signal_member_access() {
        use crate::inlined_fn::should_wrap_in_fn_signal;
        use oxc_allocator::Allocator;
        use oxc_parser::Parser;
        use oxc_span::SourceType;

        let allocator = Allocator::default();
        let source = "store.count + 1";
        let source_type = SourceType::mjs();
        let expr = Parser::new(&allocator, source, source_type)
            .parse_expression()
            .expect("Should parse expression");

        let scoped_idents = vec![("store".to_string(), ScopeId::new(0))];

        assert!(
            should_wrap_in_fn_signal(&expr, &scoped_idents),
            "Member access pattern should need _fnSignal wrapping"
        );
    }

    #[test]
    fn test_should_not_wrap_simple_identifier() {
        use crate::inlined_fn::should_wrap_in_fn_signal;
        use oxc_allocator::Allocator;
        use oxc_parser::Parser;
        use oxc_span::SourceType;

        let allocator = Allocator::default();
        let source = "count + 1";
        let source_type = SourceType::mjs();
        let expr = Parser::new(&allocator, source, source_type)
            .parse_expression()
            .expect("Should parse expression");

        let scoped_idents = vec![("count".to_string(), ScopeId::new(0))];

        assert!(
            !should_wrap_in_fn_signal(&expr, &scoped_idents),
            "Simple identifier should NOT need _fnSignal wrapping"
        );
    }

    #[test]
    fn test_should_not_wrap_arrow_function() {
        use crate::inlined_fn::should_wrap_in_fn_signal;
        use oxc_allocator::Allocator;
        use oxc_parser::Parser;
        use oxc_span::SourceType;

        let allocator = Allocator::default();
        let source = "() => store.count";
        let source_type = SourceType::mjs();
        let expr = Parser::new(&allocator, source, source_type)
            .parse_expression()
            .expect("Should parse expression");

        let scoped_idents = vec![("store".to_string(), ScopeId::new(0))];

        assert!(
            !should_wrap_in_fn_signal(&expr, &scoped_idents),
            "Arrow function should NOT need _fnSignal wrapping"
        );
    }

    #[test]
    fn test_should_not_wrap_call_expression() {
        use crate::inlined_fn::should_wrap_in_fn_signal;
        use oxc_allocator::Allocator;
        use oxc_parser::Parser;
        use oxc_span::SourceType;

        let allocator = Allocator::default();
        let source = "store.count + calculate()";
        let source_type = SourceType::mjs();
        let expr = Parser::new(&allocator, source, source_type)
            .parse_expression()
            .expect("Should parse expression");

        let scoped_idents = vec![("store".to_string(), ScopeId::new(0))];

        assert!(
            !should_wrap_in_fn_signal(&expr, &scoped_idents),
            "Expression with call should NOT need _fnSignal wrapping"
        );
    }

    // ==================== convert_inlined_fn Tests (2 tests) ====================

    #[test]
    fn test_convert_inlined_fn_basic() {
        use crate::inlined_fn::convert_inlined_fn;
        use oxc_allocator::Allocator;
        use oxc_ast::AstBuilder;
        use oxc_parser::Parser;
        use oxc_span::SourceType;

        let allocator = Allocator::default();
        let source = "store.count + 1";
        let source_type = SourceType::mjs();
        let expr = Parser::new(&allocator, source, source_type)
            .parse_expression()
            .expect("Should parse expression");
        let builder = AstBuilder::new(&allocator);

        let scoped_idents = vec![("store".to_string(), ScopeId::new(0))];

        let result = convert_inlined_fn(&expr, &scoped_idents, 0, &builder, &allocator);

        assert!(
            result.is_some(),
            "Should produce InlinedFnResult for member access expression"
        );

        let result = result.unwrap();
        assert_eq!(result.hoisted_name, "_hf0");
        assert!(!result.captures.is_empty(), "Should have captures");
    }

    #[test]
    fn test_convert_inlined_fn_no_scoped_idents() {
        use crate::inlined_fn::convert_inlined_fn;
        use oxc_allocator::Allocator;
        use oxc_ast::AstBuilder;
        use oxc_parser::Parser;
        use oxc_span::SourceType;

        let allocator = Allocator::default();
        let source = "store.count + 1";
        let source_type = SourceType::mjs();
        let expr = Parser::new(&allocator, source, source_type)
            .parse_expression()
            .expect("Should parse expression");
        let builder = AstBuilder::new(&allocator);

        let scoped_idents: Vec<(String, ScopeId)> = vec![];

        let result = convert_inlined_fn(&expr, &scoped_idents, 0, &builder, &allocator);

        assert!(result.is_none(), "Should return None when no scoped idents");
    }

    // ==================== is_bind_directive Test (1 test) ====================

    #[test]
    fn test_is_bind_directive_helper() {
        use crate::transform::is_bind_directive;
        assert_eq!(is_bind_directive("bind:value"), Some(false));
        assert_eq!(is_bind_directive("bind:checked"), Some(true));
        assert_eq!(is_bind_directive("bind:stuff"), None);
        assert_eq!(is_bind_directive("onClick$"), None);
        assert_eq!(is_bind_directive("value"), None);
    }
}
