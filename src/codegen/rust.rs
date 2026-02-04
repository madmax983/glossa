//! Rust code generation
//!
//! This module handles the final step of the compilation pipeline: translating
//! the Analyzed Semantic Model into executable Rust code.
//!
//! # The CodeGen Strategy
//!
//! Instead of concatenating strings (which is error-prone and unsafe), we use the
//! [`quote`] crate to construct a valid Rust TokenStream. This ensures that the
//! generated code is syntactically correct and handles edge cases like escaping strings.
//!
//! # Name Sanitization
//!
//! Ancient Greek characters cannot be used directly as Rust identifiers (mostly).
//! This module performs **Transliteration** to convert Greek names into valid ASCII Rust identifiers.
//!
//! * `ξ` (xi) -> `xi`
//! * `χρήστος` -> `chrestos`
//! * `λόγος` -> `logos`
//!
//! See [`sanitize_name`] for details.

use crate::morphology::lexicon::{BinaryOp, UnaryOp};
use crate::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedIteratorOp, AnalyzedMethod, AnalyzedProgram,
    AnalyzedStatement, GlossaType, StatementKind,
};
use crate::text::normalize_greek;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Generate Rust code from an Analyzed Program
pub fn generate_rust(program: &AnalyzedProgram) -> String {
    // Check if we need collection imports
    let needs_collections = uses_collections(program);

    // Separate trait defs, struct defs, trait impls, function defs, and main body statements
    let mut trait_defs = Vec::new();
    let mut struct_defs = Vec::new();
    let mut trait_impls = Vec::new();
    let mut fn_defs = Vec::new();
    let mut main_stmts = Vec::new();

    for stmt in &program.statements {
        match &stmt.kind {
            StatementKind::TraitDefinition { .. } => trait_defs.push(generate_statement(stmt)),
            StatementKind::TypeDefinition { .. } => struct_defs.push(generate_statement(stmt)),
            StatementKind::TraitImplementation { .. } => trait_impls.push(generate_statement(stmt)),
            StatementKind::FunctionDef { .. } => fn_defs.push(generate_statement(stmt)),
            _ => main_stmts.push(generate_statement(stmt)),
        }
    }

    // Generate imports if needed
    let imports = if needs_collections {
        quote! { use std::collections::{HashMap, HashSet}; }
    } else {
        quote! {}
    };

    let output = quote! {
        #imports

        #(#trait_defs)*

        #(#struct_defs)*

        #(#trait_impls)*

        #(#fn_defs)*

        fn main() {
            #(#main_stmts)*
        }
    };

    output.to_string()
}

/// Check if the program uses collection types (HashMap, HashSet)
fn uses_collections(program: &AnalyzedProgram) -> bool {
    for stmt in &program.statements {
        if statement_uses_collections(stmt) {
            return true;
        }
    }
    false
}

/// Check if a statement uses collection types
fn statement_uses_collections(stmt: &AnalyzedStatement) -> bool {
    match &stmt.kind {
        StatementKind::Binding { value_type: _, .. }
        | StatementKind::Assignment { value_type: _, .. } => {
            // Also check expressions
            stmt.expressions.iter().any(expr_uses_collections)
        }
        StatementKind::Print | StatementKind::Query => {
            stmt.expressions.iter().any(expr_uses_collections)
        }
        StatementKind::Expression => stmt.expressions.iter().any(expr_uses_collections),
        StatementKind::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_uses_collections(condition)
                || then_body.iter().any(statement_uses_collections)
                || else_body
                    .as_ref()
                    .map(|b| b.iter().any(statement_uses_collections))
                    .unwrap_or(false)
        }
        StatementKind::While { condition, body } => {
            expr_uses_collections(condition) || body.iter().any(statement_uses_collections)
        }
        StatementKind::For { iterator, body, .. } => {
            expr_uses_collections(iterator) || body.iter().any(statement_uses_collections)
        }
        StatementKind::Match { scrutinee, arms } => {
            expr_uses_collections(scrutinee)
                || arms.iter().any(|(pat, body)| {
                    expr_uses_collections(pat) || body.iter().any(statement_uses_collections)
                })
        }
        StatementKind::FunctionDef { body, .. } => body.iter().any(statement_uses_collections),
        StatementKind::TraitImplementation { methods, .. }
        | StatementKind::TraitDefinition { methods, .. } => methods.iter().any(|m| {
            m.body
                .as_ref()
                .map(|b| b.iter().any(statement_uses_collections))
                .unwrap_or(false)
        }),
        _ => false,
    }
}

/// Check if an expression uses collection types
fn expr_uses_collections(expr: &AnalyzedExpr) -> bool {
    match &expr.expr {
        AnalyzedExprKind::CollectionNew { .. } | AnalyzedExprKind::CollectionContains { .. } => {
            true
        }
        AnalyzedExprKind::MethodCall { receiver, args, .. } => {
            expr_uses_collections(receiver) || args.iter().any(expr_uses_collections)
        }
        AnalyzedExprKind::BinOp { left, right, .. } => {
            expr_uses_collections(left) || expr_uses_collections(right)
        }
        AnalyzedExprKind::IndexAccess { array, index } => {
            expr_uses_collections(array) || expr_uses_collections(index)
        }
        // Check types inside Struct Instantiation if needed, but usually explicit new/contains is enough
        _ => false,
    }
}

/// Generate a complete Rust file with proper formatting
pub fn generate_rust_file(program: &AnalyzedProgram) -> String {
    let code = generate_rust(program);

    // Add a comment header
    format!(
        "// Generated by ΓΛΩΣΣΑ compiler\n// Αὐτόματος κῶδιξ ἀπὸ ΓΛΩΣΣΑ\n\n{}",
        code
    )
}

fn generate_statement(stmt: &AnalyzedStatement) -> TokenStream {
    match &stmt.kind {
        StatementKind::Binding { name, mutable, .. } => {
            // Get the value expression (second expression in the list, first is usually name or type info which is unused here)
            // Wait, AnalyzedStatement.expressions structure depends on how it was built.
            // Semantic analyzer puts value at index 1 for Binding (index 0 is name/type).
            let value = if stmt.expressions.len() > 1 {
                &stmt.expressions[1]
            } else {
                // Should not happen in valid program, but provide fallback
                let name_ident = format_ident!("{}", sanitize_name(name));
                return quote! { let #name_ident = 0; };
            };

            // Check if it's an array to force mutable
            let is_array = matches!(value.expr, AnalyzedExprKind::ArrayLiteral(_));
            generate_let(name, value, *mutable || is_array)
        }

        StatementKind::Assignment { name, .. } => {
            if stmt.expressions.len() > 1 {
                let name_ident = format_ident!("{}", sanitize_name(name));
                let value_tokens = generate_expr(&stmt.expressions[1]);
                quote! { #name_ident = #value_tokens; }
            } else {
                quote! {}
            }
        }

        StatementKind::Print | StatementKind::Query => generate_print(&stmt.expressions),

        StatementKind::Expression => {
            if let Some(first) = stmt.expressions.first() {
                let expr_tokens = generate_expr(first);
                quote! { #expr_tokens; }
            } else {
                quote! {}
            }
        }

        StatementKind::If {
            condition,
            then_body,
            else_body,
        } => generate_if(condition, then_body, else_body),

        StatementKind::While { condition, body } => generate_while(condition, body),

        StatementKind::For {
            variable,
            iterator,
            body,
        } => generate_for(variable, iterator, body),

        StatementKind::Match { scrutinee, arms } => generate_match(scrutinee, arms),

        StatementKind::Break => quote! { break; },

        StatementKind::Continue => quote! { continue; },

        StatementKind::Return { value } => match value {
            Some(e) => {
                let value_tokens = generate_expr(e);
                quote! { return #value_tokens; }
            }
            None => quote! { return; },
        },

        StatementKind::FunctionDef {
            name,
            params,
            body,
            return_type,
        } => generate_fn_def(name, params, body, return_type),

        StatementKind::TypeDefinition { name, fields } => generate_struct_def(name, fields),

        StatementKind::TraitDefinition { name, methods } => generate_trait_def(name, methods),

        StatementKind::TraitImplementation {
            trait_name,
            type_name,
            methods,
        } => generate_trait_impl(trait_name, type_name, methods),
    }
}

fn generate_let(name: &str, value: &AnalyzedExpr, mutable: bool) -> TokenStream {
    let name_ident = format_ident!("{}", sanitize_name(name));
    let value_tokens = generate_expr(value);

    if mutable {
        quote! { let mut #name_ident = #value_tokens; }
    } else {
        quote! { let #name_ident = #value_tokens; }
    }
}

fn generate_print(args: &[AnalyzedExpr]) -> TokenStream {
    if args.is_empty() {
        quote! { println!(); }
    } else if args.len() == 1 {
        let arg = generate_expr(&args[0]);
        // Use Display formatting
        quote! { println!("{}", #arg); }
    } else {
        // Multiple args - join with space
        let arg_tokens: Vec<TokenStream> = args.iter().map(generate_expr).collect();
        quote! { println!("{}", vec![#(format!("{}", #arg_tokens)),*].join(" ")); }
    }
}

fn generate_if(
    condition: &AnalyzedExpr,
    then_body: &[AnalyzedStatement],
    else_body: &Option<Vec<AnalyzedStatement>>,
) -> TokenStream {
    let cond = generate_expr(condition);
    let then_stmts: Vec<TokenStream> = then_body.iter().map(generate_statement).collect();

    match else_body {
        Some(else_stmts) => {
            let else_tokens: Vec<TokenStream> = else_stmts.iter().map(generate_statement).collect();
            quote! {
                if #cond {
                    #(#then_stmts)*
                } else {
                    #(#else_tokens)*
                }
            }
        }
        None => {
            quote! {
                if #cond {
                    #(#then_stmts)*
                }
            }
        }
    }
}

fn generate_while(condition: &AnalyzedExpr, body: &[AnalyzedStatement]) -> TokenStream {
    let cond = generate_expr(condition);
    let body_stmts: Vec<TokenStream> = body.iter().map(generate_statement).collect();
    quote! {
        while #cond {
            #(#body_stmts)*
        }
    }
}

fn generate_for(
    variable: &str,
    iterator: &AnalyzedExpr,
    body: &[AnalyzedStatement],
) -> TokenStream {
    let var_ident = format_ident!("{}", sanitize_name(variable));
    let iter = generate_expr(iterator);
    let body_stmts: Vec<TokenStream> = body.iter().map(generate_statement).collect();
    quote! {
        for #var_ident in #iter {
            #(#body_stmts)*
        }
    }
}

fn generate_match(
    scrutinee: &AnalyzedExpr,
    arms: &[(AnalyzedExpr, Vec<AnalyzedStatement>)],
) -> TokenStream {
    let scrut = generate_expr(scrutinee);
    let arm_tokens: Vec<TokenStream> = arms
        .iter()
        .map(|(pattern, body)| {
            // Check if pattern is wildcard (represented as BooleanLiteral(true))
            let pat = if matches!(pattern.expr, AnalyzedExprKind::BooleanLiteral(true)) {
                // Generate wildcard pattern
                quote! { _ }
            } else {
                generate_expr(pattern)
            };
            let body_stmts: Vec<TokenStream> = body.iter().map(generate_statement).collect();
            quote! { #pat => { #(#body_stmts)* } }
        })
        .collect();
    quote! {
        match #scrut {
            #(#arm_tokens),*
        }
    }
}

fn generate_fn_def(
    name: &str,
    params: &[(smol_str::SmolStr, Option<GlossaType>)],
    body: &[AnalyzedStatement],
    return_type: &Option<GlossaType>,
) -> TokenStream {
    let fn_name = format_ident!("{}", sanitize_name(name));
    let body_stmts: Vec<TokenStream> = body.iter().map(generate_statement).collect();

    // Generate parameter list
    let param_tokens: Vec<TokenStream> = params
        .iter()
        .map(|(param_name, param_type)| {
            let param_ident = format_ident!("{}", sanitize_name(param_name));
            if let Some(ty) = param_type {
                let ty_str = type_to_rust_string(ty);
                let ty_ident = format_ident!("{}", ty_str);
                quote! { #param_ident: #ty_ident }
            } else {
                quote! { #param_ident }
            }
        })
        .collect();

    // Generate return type
    if let Some(ret_type) = return_type {
        let ret_str = type_to_rust_string(ret_type);
        let ret_ty = format_ident!("{}", ret_str);
        quote! {
            fn #fn_name(#(#param_tokens),*) -> #ret_ty {
                #(#body_stmts)*
            }
        }
    } else {
        quote! {
            fn #fn_name(#(#param_tokens),*) {
                #(#body_stmts)*
            }
        }
    }
}

fn generate_struct_def(name: &str, fields: &[(smol_str::SmolStr, GlossaType)]) -> TokenStream {
    // Capitalize struct name for Rust conventions
    let struct_name = format_ident!("{}", capitalize(&sanitize_name(name)));

    // Generate field list
    let field_tokens: Vec<TokenStream> = fields
        .iter()
        .map(|(field_name, field_type)| {
            let field_ident = format_ident!("{}", sanitize_name(field_name));
            let type_str = type_to_rust_string(field_type);
            let type_ident = format_ident!("{}", type_str);
            quote! { #field_ident: #type_ident }
        })
        .collect();

    quote! {
        #[derive(Debug, Clone)]
        struct #struct_name {
            #(#field_tokens),*
        }
    }
}

fn generate_trait_def(name: &str, methods: &[AnalyzedMethod]) -> TokenStream {
    // Capitalize trait name for Rust conventions
    let trait_name = format_ident!("{}", capitalize(&sanitize_name(name)));

    // Generate method signatures
    let method_tokens: Vec<TokenStream> = methods
        .iter()
        .map(|method| {
            let method_name = format_ident!("{}", sanitize_name(&method.name));

            // Generate parameter list
            let param_tokens: Vec<TokenStream> = method
                .params
                .iter()
                .enumerate()
                .map(|(idx, (param_name, param_type))| {
                    // Special case: first parameter named "self" becomes &self
                    if is_self_parameter(param_name, idx) {
                        quote! { &self }
                    } else {
                        let param_ident = format_ident!("{}", sanitize_name(param_name));
                        let type_str = type_to_rust_string(param_type);
                        let ty = format_ident!("{}", type_str);
                        quote! { #param_ident: #ty }
                    }
                })
                .collect();

            // Generate return type
            if let Some(ret_type) = &method.return_type {
                let ret_str = type_to_rust_string(ret_type);
                let ret_ty = format_ident!("{}", ret_str);

                if let Some(body) = &method.body {
                    let body_stmts: Vec<TokenStream> =
                        body.iter().map(generate_statement).collect();
                    quote! {
                        fn #method_name(#(#param_tokens),*) -> #ret_ty {
                            #(#body_stmts)*
                        }
                    }
                } else {
                    quote! {
                        fn #method_name(#(#param_tokens),*) -> #ret_ty;
                    }
                }
            } else if let Some(body) = &method.body {
                let body_stmts: Vec<TokenStream> =
                    body.iter().map(generate_statement).collect();
                quote! {
                    fn #method_name(#(#param_tokens),*) {
                        #(#body_stmts)*
                    }
                }
            } else {
                quote! {
                    fn #method_name(#(#param_tokens),*);
                }
            }
        })
        .collect();

    quote! {
        trait #trait_name {
            #(#method_tokens)*
        }
    }
}

fn generate_trait_impl(
    trait_name: &str,
    type_name: &str,
    methods: &[AnalyzedMethod],
) -> TokenStream {
    // Capitalize trait and type names for Rust conventions
    let trait_ident = format_ident!("{}", capitalize(&sanitize_name(trait_name)));
    let type_ident = format_ident!("{}", capitalize(&sanitize_name(type_name)));

    // Generate method implementations
    let method_tokens: Vec<TokenStream> = methods
        .iter()
        .map(|method| {
            let method_name = format_ident!("{}", sanitize_name(&method.name));

            // Generate parameter list
            let param_tokens: Vec<TokenStream> = method
                .params
                .iter()
                .enumerate()
                .map(|(idx, (param_name, param_type))| {
                    if is_self_parameter(param_name, idx) {
                        quote! { &self }
                    } else {
                        let param_ident = format_ident!("{}", sanitize_name(param_name));
                        let type_str = type_to_rust_string(param_type);
                        let ty = format_ident!("{}", type_str);
                        quote! { #param_ident: #ty }
                    }
                })
                .collect();

            // Generate method body
            let body_stmts: Vec<TokenStream> = if let Some(body) = &method.body {
                body.iter().map(generate_statement).collect()
            } else {
                Vec::new()
            };

            // Generate return type
            if let Some(ret_type) = &method.return_type {
                let ret_str = type_to_rust_string(ret_type);
                let ret_ty = format_ident!("{}", ret_str);
                quote! {
                    fn #method_name(#(#param_tokens),*) -> #ret_ty {
                        #(#body_stmts)*
                    }
                }
            } else {
                quote! {
                    fn #method_name(#(#param_tokens),*) {
                        #(#body_stmts)*
                    }
                }
            }
        })
        .collect();

    quote! {
        impl #trait_ident for #type_ident {
            #(#method_tokens)*
        }
    }
}

fn generate_expr(expr: &AnalyzedExpr) -> TokenStream {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => {
            quote! { #s }
        }

        AnalyzedExprKind::NumberLiteral(n) => {
            quote! { #n }
        }

        AnalyzedExprKind::Literal(n) => {
            quote! { #n }
        }

        AnalyzedExprKind::BooleanLiteral(b) => {
            quote! { #b }
        }

        AnalyzedExprKind::ArrayLiteral(elements) => {
            let elem_tokens: Vec<TokenStream> = elements.iter().map(generate_expr).collect();
            quote! { vec![#(#elem_tokens),*] }
        }

        AnalyzedExprKind::Some(inner) => {
            let inner_tokens = generate_expr(inner);
            quote! { Some(#inner_tokens) }
        }

        AnalyzedExprKind::None => {
            quote! { None }
        }

        AnalyzedExprKind::Ok(inner) => {
            let inner_tokens = generate_expr(inner);
            quote! { Ok(#inner_tokens) }
        }

        AnalyzedExprKind::Err(inner) => {
            let inner_tokens = generate_expr(inner);
            quote! { Err(#inner_tokens) }
        }

        AnalyzedExprKind::Try(inner) => {
            let inner_tokens = generate_expr(inner);
            quote! { #inner_tokens? }
        }

        AnalyzedExprKind::Unwrap(inner) => {
            let inner_tokens = generate_expr(inner);
            quote! { #inner_tokens.unwrap() }
        }

        AnalyzedExprKind::IndexAccess { array, index } => {
            let array_tokens = generate_expr(array);
            let index_tokens = generate_expr(index);
            // Safety check for negative index
            quote! {
                {
                    let idx = #index_tokens;
                    if idx < 0 {
                        panic!("Negative index access: {}", idx);
                    }
                    #array_tokens[idx as usize]
                }
            }
        }

        AnalyzedExprKind::Variable(name) => {
            let name_ident = format_ident!("{}", sanitize_name(name));
            quote! { #name_ident }
        }

        AnalyzedExprKind::PropertyAccess { owner, property } => {
            let obj = generate_expr(owner);
            let field_ident = format_ident!("{}", sanitize_name(property));
            quote! { #obj.#field_ident }
        }

        AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } => {
            let recv = generate_expr(receiver);
            let method_ident = format_ident!("{}", sanitize_name(method));
            let arg_tokens: Vec<TokenStream> = args.iter().map(generate_expr).collect();
            quote! { #recv.#method_ident(#(#arg_tokens),*) }
        }

        AnalyzedExprKind::TraitMethodCall {
            receiver,
            method_name,
            args,
            ..
        } => {
            // Treat as regular method call
            let recv = generate_expr(receiver);
            let method_ident = format_ident!("{}", sanitize_name(method_name));
            let arg_tokens: Vec<TokenStream> = args.iter().map(generate_expr).collect();
            quote! { #recv.#method_ident(#(#arg_tokens),*) }
        }

        AnalyzedExprKind::VerbCall { verb, args }
        | AnalyzedExprKind::FunctionCall { func: verb, args } => {
            let func_ident = format_ident!("{}", sanitize_name(verb));
            let arg_tokens: Vec<TokenStream> = args.iter().map(generate_expr).collect();
            quote! { #func_ident(#(#arg_tokens),*) }
        }

        AnalyzedExprKind::BinOp { op, left, right } => generate_bin_op(*op, left, right),

        AnalyzedExprKind::UnaryOp { op, operand } => {
            match op {
                UnaryOp::Not => {
                    let operand_tokens = generate_expr(operand);
                    // Standard Rust logical not or bitwise not
                    quote! { !#operand_tokens }
                }
                UnaryOp::Neg => {
                    let operand_tokens = generate_expr(operand);
                    quote! { -#operand_tokens }
                }
            }
        }

        AnalyzedExprKind::Range {
            start,
            end,
            inclusive,
        } => {
            let start_tokens = generate_expr(start);
            let end_tokens = generate_expr(end);
            if *inclusive {
                quote! { (#start_tokens..=#end_tokens) }
            } else {
                quote! { (#start_tokens..#end_tokens) }
            }
        }

        AnalyzedExprKind::StructInstantiation {
            type_name,
            fields,
            args,
        } => generate_struct_lit(type_name, fields, args),

        AnalyzedExprKind::Lambda {
            params,
            body,
            capture_mode,
        } => generate_closure(params, body, capture_mode),

        AnalyzedExprKind::IteratorChain { collection, ops } => {
            generate_iterator_chain(collection, ops)
        }

        AnalyzedExprKind::CollectionNew { collection_type } => {
            // Generate HashSet::new() or HashMap::new()
            let type_ident = format_ident!("{}", collection_type);
            quote! { #type_ident::new() }
        }

        AnalyzedExprKind::CollectionContains {
            collection,
            element,
            is_map,
        } => {
            let coll = generate_expr(collection);
            let elem = generate_expr(element);
            if *is_map {
                // HashMap uses .contains_key(&key)
                quote! { #coll.contains_key(&#elem) }
            } else {
                match &element.expr {
                    // When the element is a string literal, `generate_expr`
                    // already yields a `&str`, so we call `.contains(elem)`
                    // without adding another `&`.
                    AnalyzedExprKind::StringLiteral(_) => {
                        quote! { #coll.contains(#elem) }
                    }
                    // In all other cases, keep the existing behavior and
                    // pass a reference to the element.
                    _ => {
                        // HashSet uses .contains(&element)
                        quote! { #coll.contains(&#elem) }
                    }
                }
            }
        }
    }
}

fn generate_bin_op(op: BinaryOp, left: &AnalyzedExpr, right: &AnalyzedExpr) -> TokenStream {
    let left_tokens = generate_expr(left);
    let right_tokens = generate_expr(right);

    match op {
        BinaryOp::Add => quote! { (#left_tokens + #right_tokens) },
        BinaryOp::Sub => quote! { (#left_tokens - #right_tokens) },
        BinaryOp::Mul => quote! { (#left_tokens * #right_tokens) },
        BinaryOp::Div => quote! { (#left_tokens / #right_tokens) },
        BinaryOp::Mod => quote! { (#left_tokens % #right_tokens) },
        BinaryOp::Eq => quote! { (#left_tokens == #right_tokens) },
        BinaryOp::Ne => quote! { (#left_tokens != #right_tokens) },
        BinaryOp::Lt => quote! { (#left_tokens < #right_tokens) },
        BinaryOp::Le => quote! { (#left_tokens <= #right_tokens) },
        BinaryOp::Gt => quote! { (#left_tokens > #right_tokens) },
        BinaryOp::Ge => quote! { (#left_tokens >= #right_tokens) },
        BinaryOp::And => quote! { (#left_tokens && #right_tokens) },
        BinaryOp::Or => quote! { (#left_tokens || #right_tokens) },
    }
}

fn generate_struct_lit(
    type_name: &str,
    fields: &[smol_str::SmolStr],
    args: &[AnalyzedExpr],
) -> TokenStream {
    // Capitalize struct name for Rust conventions
    let struct_name = format_ident!("{}", capitalize(&sanitize_name(type_name)));

    // Generate field: value pairs using actual field names
    let field_assignments: Vec<TokenStream> = fields
        .iter()
        .zip(args.iter())
        .map(|(field_name, arg)| {
            let field_ident = format_ident!("{}", sanitize_name(field_name));
            let arg_token = generate_expr(arg);
            quote! { #field_ident: #arg_token }
        })
        .collect();

    quote! { #struct_name { #(#field_assignments),* } }
}

fn generate_closure(
    params: &[smol_str::SmolStr],
    body: &AnalyzedExpr,
    capture_mode: &crate::ast::CaptureMode,
) -> TokenStream {
    use crate::ast::CaptureMode;

    let body_tokens = generate_expr(body);
    let params_idents: Vec<_> = params
        .iter()
        .map(|p| format_ident!("{}", sanitize_name(p)))
        .collect();

    match capture_mode {
        CaptureMode::Move => {
            quote! { move |#(#params_idents),*| #body_tokens }
        }
        CaptureMode::Borrow => {
            quote! { |#(#params_idents),*| #body_tokens }
        }
        CaptureMode::Memoize => {
            // Perfect participle: lazy evaluation with caching
            quote! {
                {
                    use std::cell::RefCell;
                    let cache = RefCell::new(None);
                    move |#(#params_idents),*| {
                        let mut cache_ref = cache.borrow_mut();
                        if cache_ref.is_none() {
                            *cache_ref = Some(#body_tokens);
                        }
                        cache_ref.clone().unwrap()
                    }
                }
            }
        }
    }
}

fn generate_iterator_chain(collection: &AnalyzedExpr, ops: &[AnalyzedIteratorOp]) -> TokenStream {
    let mut current = generate_expr(collection);

    for op in ops {
        current = match op {
            AnalyzedIteratorOp::Iter => {
                quote! { #current.iter() }
            }
            AnalyzedIteratorOp::Map(closure) => {
                let closure_tokens = generate_expr(closure);
                quote! { #current.map(#closure_tokens) }
            }
            AnalyzedIteratorOp::Filter(closure) => {
                let closure_tokens = generate_expr(closure);
                quote! { #current.filter(#closure_tokens) }
            }
            AnalyzedIteratorOp::Find(closure) => {
                let closure_tokens = generate_expr(closure);
                quote! { #current.find(#closure_tokens) }
            }
            AnalyzedIteratorOp::Fold { init, closure } => {
                let init_tokens = generate_expr(init);
                let closure_tokens = generate_expr(closure);
                quote! { #current.fold(#init_tokens, #closure_tokens) }
            }
            AnalyzedIteratorOp::Any(closure) => {
                let closure_tokens = generate_expr(closure);
                quote! { #current.any(#closure_tokens) }
            }
            AnalyzedIteratorOp::All(closure) => {
                let closure_tokens = generate_expr(closure);
                quote! { #current.all(#closure_tokens) }
            }
            AnalyzedIteratorOp::Collect => {
                quote! { #current.collect() }
            }
        };
    }

    current
}

/// Capitalize the first letter of a string (for Rust type/trait names)
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Check if a parameter represents the self parameter in a trait method
fn is_self_parameter(param_name: &str, idx: usize) -> bool {
    if idx != 0 {
        return false;
    }
    let normalized = normalize_greek(param_name);
    // Check for "self", "τω" (normalized from τῷ), or if param name contains "self"
    normalized == "self"
        || param_name == "self"
        || normalized == "τω"
        || param_name.contains("self")
}

/// Sanitize a Greek name for use as a Rust identifier
fn sanitize_name(name: &str) -> String {
    // Map single Greek letters to their names
    if name.len() <= 2 {
        // Could be a single Greek letter (2 bytes for UTF-8)
        match name {
            "α" => return "alpha".to_string(),
            "β" => return "beta".to_string(),
            "γ" => return "gamma".to_string(),
            "δ" => return "delta".to_string(),
            "ε" => return "epsilon".to_string(),
            "ζ" => return "zeta".to_string(),
            "η" => return "eta".to_string(),
            "θ" => return "theta".to_string(),
            "ι" => return "iota".to_string(),
            "κ" => return "kappa".to_string(),
            "λ" => return "lambda".to_string(),
            "μ" => return "mu".to_string(),
            "ν" => return "nu".to_string(),
            "ξ" => return "xi".to_string(),
            "ο" => return "omicron".to_string(),
            "π" => return "pi".to_string(),
            "ρ" => return "rho".to_string(),
            "σ" | "ς" => return "sigma".to_string(),
            "τ" => return "tau".to_string(),
            "υ" => return "upsilon".to_string(),
            "φ" => return "phi".to_string(),
            "χ" => return "chi".to_string(),
            "ψ" => return "psi".to_string(),
            "ω" => return "omega".to_string(),
            _ => {}
        }
    }

    // Transliterate the full name
    transliterate(name)
}

/// Transliterate Greek to Latin characters
fn transliterate(greek: &str) -> String {
    let mut result = String::new();

    for c in greek.chars() {
        let trans = match c {
            'α' => "a",
            'β' => "b",
            'γ' => "g",
            'δ' => "d",
            'ε' => "e",
            'ζ' => "z",
            'η' => "e",
            'θ' => "th",
            'ι' => "i",
            'κ' => "k",
            'λ' => "l",
            'μ' => "m",
            'ν' => "n",
            'ξ' => "x",
            'ο' => "o",
            'π' => "p",
            'ρ' => "r",
            'σ' | 'ς' => "s",
            'τ' => "t",
            'υ' => "u",
            'φ' => "ph",
            'χ' => "ch",
            'ψ' => "ps",
            'ω' => "o",
            _ => {
                // Keep only ASCII alphanumeric characters and underscore
                if c.is_ascii_alphanumeric() || c == '_' {
                    result.push(c);
                } else {
                    // Replace invalid characters with unique hex code to prevent collisions
                    // e.g. ϟ -> _u3df_
                    use std::fmt::Write;
                    write!(result, "_u{:x}_", c as u32).unwrap();
                }
                continue;
            }
        };
        result.push_str(trans);
    }

    // Ensure it starts with a letter or underscore (valid Rust identifier)
    if result
        .chars()
        .next()
        .map(|c| c.is_numeric())
        .unwrap_or(true)
    {
        format!("_{}", result)
    } else {
        result
    }
}

/// Helper to get Rust type string, working around GlossaType::to_rust issues
fn type_to_rust_string(ty: &GlossaType) -> String {
    match ty {
        GlossaType::Struct { name, .. } => capitalize(&sanitize_name(name)),
        _ => ty.to_rust(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    fn compile(source: &str) -> String {
        let ast = parse(source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        generate_rust(&analyzed)
    }

    #[test]
    fn test_generate_hello() {
        let code = compile("«χαῖρε» λέγε.");
        // quote! generates `println !` with space
        assert!(code.contains("println"), "Expected println in: {}", code);
        assert!(code.contains("χαῖρε"));
    }

    #[test]
    fn test_generate_binding() {
        let code = compile("ξ πέντε ἔστω.");
        assert!(code.contains("let xi"));
        assert!(code.contains("5"));
    }

    #[test]
    fn test_generate_number() {
        let code = compile("42 λέγε.");
        assert!(code.contains("println"), "Expected println in: {}", code);
        assert!(code.contains("42"));
    }

    #[test]
    fn test_sanitize_greek_letter() {
        assert_eq!(sanitize_name("ξ"), "xi");
        assert_eq!(sanitize_name("α"), "alpha");
        assert_eq!(sanitize_name("ω"), "omega");
    }

    #[test]
    fn test_transliterate() {
        assert_eq!(transliterate("χρηστος"), "chrestos");
        assert_eq!(transliterate("λογος"), "logos");
        assert_eq!(transliterate("φιλοσοφια"), "philosophia");
    }

    #[test]
    fn test_transliterate_unique() {
        // Test that different invalid characters produce different outputs
        let koppa = "ϟ";
        let stigma = "ϛ";

        let t_koppa = transliterate(koppa);
        let t_stigma = transliterate(stigma);

        assert_ne!(
            t_koppa, t_stigma,
            "Different invalid chars should not collide"
        );
        assert!(t_koppa.contains("_u3df_")); // Koppa is 0x3DF
        assert!(t_stigma.contains("_u3db_")); // Stigma is 0x3DB
    }

    #[test]
    fn test_transliterate_mixed_valid_invalid() {
        // Test mixing valid and invalid characters
        let input = "αϟβ";
        let output = transliterate(input);
        assert_eq!(output, "a_u3df_b");
    }

    #[test]
    fn test_generate_full_program() {
        let code = compile("ξ πέντε ἔστω. ξ λέγε.");
        assert!(code.contains("let xi = 5"), "Expected binding in: {}", code);
        assert!(code.contains("println"), "Expected println in: {}", code);
    }
}
