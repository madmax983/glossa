use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, AnalyzedMethod, GlossaType};
use crate::text::normalize_greek;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use super::utils::{sanitize_name, capitalize};
use super::types::to_rust_type;
use super::expressions::{generate_expr, expr_uses_collections};

/// Generate Rust code for a single analyzed statement
pub fn generate_statement_code(stmt: &AnalyzedStatement) -> String {
    generate_statement(stmt).to_string()
}

pub(crate) fn generate_statement(stmt: &AnalyzedStatement) -> TokenStream {
    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            // Check if it's an array to force mutable
            let is_array = matches!(value.expr, AnalyzedExprKind::ArrayLiteral(_));
            generate_let(name, value, *mutable || is_array)
        }

        AnalyzedStatement::Assignment { name, value } => {
            let name_ident = format_ident!("{}", sanitize_name(name));
            let value_tokens = generate_expr(value);
            quote! { #name_ident = #value_tokens; }
        }

        AnalyzedStatement::Print(exprs) | AnalyzedStatement::Query(exprs) => generate_print(exprs),

        AnalyzedStatement::Expression(exprs) => {
            let expr_tokens: Vec<TokenStream> = exprs
                .iter()
                .map(|e| {
                    let tokens = generate_expr(e);
                    quote! { #tokens; }
                })
                .collect();
            quote! { #(#expr_tokens)* }
        }

        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => generate_if(condition, then_body, else_body),

        AnalyzedStatement::While { condition, body } => generate_while(condition, body),

        AnalyzedStatement::For {
            variable,
            iterator,
            body,
        } => generate_for(variable, iterator, body),

        AnalyzedStatement::Match { scrutinee, arms } => generate_match(scrutinee, arms),

        AnalyzedStatement::Break => quote! { break; },

        AnalyzedStatement::Continue => quote! { continue; },

        AnalyzedStatement::Return { value } => match value {
            Some(e) => {
                let value_tokens = generate_expr(e);
                quote! { return #value_tokens; }
            }
            None => quote! { return; },
        },

        AnalyzedStatement::FunctionDef {
            name,
            params,
            body,
            return_type,
        } => generate_fn_def(name, params, body, return_type),

        AnalyzedStatement::TypeDefinition { name, fields } => generate_struct_def(name, fields),

        AnalyzedStatement::TraitDefinition { name, methods } => generate_trait_def(name, methods),

        AnalyzedStatement::TraitImplementation {
            trait_name,
            type_name,
            methods,
        } => generate_trait_impl(trait_name, type_name, methods),

        AnalyzedStatement::TestDeclaration { name, body } => generate_test(name, body),
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
                let ty_str = to_rust_type(ty);
                let ty_ident = format_ident!("{}", ty_str);
                quote! { #param_ident: #ty_ident }
            } else {
                quote! { #param_ident }
            }
        })
        .collect();

    // Generate return type
    if let Some(ret_type) = return_type {
        let ret_str = to_rust_type(ret_type);
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
            let type_str = to_rust_type(field_type);
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
                        let type_str = to_rust_type(param_type);
                        let ty = format_ident!("{}", type_str);
                        quote! { #param_ident: #ty }
                    }
                })
                .collect();

            // Generate return type
            if let Some(ret_type) = &method.return_type {
                let ret_str = to_rust_type(ret_type);
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
                let body_stmts: Vec<TokenStream> = body.iter().map(generate_statement).collect();
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
                        let type_str = to_rust_type(param_type);
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
                let ret_str = to_rust_type(ret_type);
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

fn generate_test(name: &str, body: &[AnalyzedStatement]) -> TokenStream {
    // Sanitize test name for Rust function identifier
    // Replace spaces and special chars with underscores
    let test_fn_name = name
        .to_lowercase()
        .replace([' ', '-'], "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>();

    let test_ident = format_ident!("test_{}", test_fn_name);

    // Generate body statements
    let body_stmts: Vec<TokenStream> = body.iter().map(generate_statement).collect();

    quote! {
        #[test]
        fn #test_ident() {
            #(#body_stmts)*
        }
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

/// Check if a statement uses collection types
pub(crate) fn statement_uses_collections(stmt: &AnalyzedStatement) -> bool {
    match stmt {
        AnalyzedStatement::Binding { value, .. } | AnalyzedStatement::Assignment { value, .. } => {
            expr_uses_collections(value)
        }
        AnalyzedStatement::Print(exprs)
        | AnalyzedStatement::Query(exprs)
        | AnalyzedStatement::Expression(exprs) => exprs.iter().any(expr_uses_collections),
        AnalyzedStatement::If {
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
        AnalyzedStatement::While { condition, body } => {
            expr_uses_collections(condition) || body.iter().any(statement_uses_collections)
        }
        AnalyzedStatement::For { iterator, body, .. } => {
            expr_uses_collections(iterator) || body.iter().any(statement_uses_collections)
        }
        AnalyzedStatement::Match { scrutinee, arms } => {
            expr_uses_collections(scrutinee)
                || arms.iter().any(|(pat, body)| {
                    expr_uses_collections(pat) || body.iter().any(statement_uses_collections)
                })
        }
        AnalyzedStatement::FunctionDef { body, .. } => body.iter().any(statement_uses_collections),
        AnalyzedStatement::TraitImplementation { methods, .. }
        | AnalyzedStatement::TraitDefinition { methods, .. } => methods.iter().any(|m| {
            m.body
                .as_ref()
                .map(|b| b.iter().any(statement_uses_collections))
                .unwrap_or(false)
        }),
        AnalyzedStatement::TestDeclaration { body, .. } => {
            body.iter().any(statement_uses_collections)
        }
        _ => false,
    }
}
