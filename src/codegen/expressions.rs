use super::types::is_std_type;
use super::utils::{capitalize, sanitize_name};
use crate::morphology::lexicon::{BinaryOp, UnaryOp};
use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, CaptureMode, GlossaType};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn generate_expr(expr: &AnalyzedExpr) -> TokenStream {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => {
            quote! { #s }
        }

        AnalyzedExprKind::NumberLiteral(n) => {
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

            // Check if this is a standard library method call on a standard type
            let method_ident = if is_std_method(method) && is_std_type(&receiver.glossa_type) {
                // Use the raw method name (e.g., "len", "push")
                format_ident!("{}", method.as_str())
            } else {
                // Sanitize (prefix with g_) for user-defined methods
                format_ident!("{}", sanitize_name(method))
            };

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
                UnaryOp::Ref => {
                    let operand_tokens = generate_expr(operand);
                    quote! { &#operand_tokens }
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

        AnalyzedExprKind::CollectionNew { collection_type } => {
            // Generate HashSet::new() or HashMap::new()
            let type_ident = format_ident!("{}", collection_type);
            quote! { #type_ident::new() }
        }

        AnalyzedExprKind::Assert { condition } => {
            let cond = generate_expr(condition);
            quote! { assert!(#cond) }
        }

        AnalyzedExprKind::AssertEq { left, right } => {
            let left_tokens = generate_expr(left);
            let right_tokens = generate_expr(right);
            quote! { assert_eq!(#left_tokens, #right_tokens) }
        }
    }
}

fn generate_bin_op(op: BinaryOp, left: &AnalyzedExpr, right: &AnalyzedExpr) -> TokenStream {
    let left_tokens = generate_expr(left);
    let right_tokens = generate_expr(right);

    // Use checked arithmetic only for numeric types
    let use_checked = matches!(left.glossa_type, GlossaType::Number);

    match op {
        BinaryOp::Add if use_checked => {
            quote! { (#left_tokens).checked_add(#right_tokens).expect("arithmetic overflow") }
        }
        BinaryOp::Sub if use_checked => {
            quote! { (#left_tokens).checked_sub(#right_tokens).expect("arithmetic overflow") }
        }
        BinaryOp::Mul if use_checked => {
            quote! { (#left_tokens).checked_mul(#right_tokens).expect("arithmetic overflow") }
        }
        BinaryOp::Div if use_checked => {
            quote! { (#left_tokens).checked_div(#right_tokens).expect("division by zero or overflow") }
        }
        BinaryOp::Mod if use_checked => {
            quote! { (#left_tokens).checked_rem(#right_tokens).expect("division by zero or overflow") }
        }

        // Fallback or standard operators
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
    capture_mode: &CaptureMode,
) -> TokenStream {
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
            // Warden Security Check: Memoization ignores arguments, so it's only safe for 0-arity closures (thunks).
            // If we allow arguments, we risk caching incorrect results (e.g. f(1) cached, f(2) returns cached f(1)).
            if !params.is_empty() {
                panic!("Memoization is only supported for 0-argument closures");
            }

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

/// Check if a method name belongs to the Rust standard library allowlist
fn is_std_method(name: &str) -> bool {
    matches!(
        name,
        "len"
            | "push"
            | "unwrap"
            | "to_string"
            | "clone"
            | "default"
            | "into"
            | "from"
            | "eq"
            | "insert"
            | "contains"
            | "contains_key"
            | "get"
            | "remove"
            | "iter"
            | "map"
            | "filter"
            | "collect"
            | "any"
            | "all"
            | "fold"
            | "find"
            | "as_str"
            | "chars"
            | "lines"
            | "is_empty"
    )
}

/// Check if an expression uses collection types
pub(crate) fn expr_uses_collections(expr: &AnalyzedExpr) -> bool {
    match &expr.expr {
        AnalyzedExprKind::CollectionNew { .. } => true,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_unreachable_operators() {
        // Manually trigger fallback operators like Le/Ge that aren't parsed yet
        let left = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(5),
            glossa_type: GlossaType::Number,
        };
        let right = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(10),
            glossa_type: GlossaType::Number,
        };

        // Test Ge (Greater or Equal)
        let op_ge = BinaryOp::Ge;
        let tokens_ge = generate_bin_op(op_ge, &left, &right);
        let code_ge = tokens_ge.to_string();
        assert!(code_ge.contains(">="));

        // Test Le (Less or Equal)
        let op_le = BinaryOp::Le;
        let tokens_le = generate_bin_op(op_le, &left, &right);
        let code_le = tokens_le.to_string();
        assert!(code_le.contains("<="));
    }
}
