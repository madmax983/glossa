//! Code generation for ΓΛΩΣΣΑ
//!
//! This module handles the translation of the Analyzed Program into executable Rust code.
//!
//! # The Strategy: Transpilation
//!
//! ΓΛΩΣΣΑ compiles to Rust rather than LLVM IR or Assembly for several strategic reasons.
//! This approach is often called "transpilation" or "source-to-source compilation".
//!
//! ## 1. Safety First (The "Rust" Shield)
//! By targeting Rust, we inherit the most sophisticated borrow checker and memory safety
//! system in the world. If the ΓΛΩΣΣΑ compiler generates valid Rust code, the resulting
//! binary is guaranteed to be memory-safe (no segfaults, no data races).
//!
//! ## 2. The Ecosystem
//! Users get instant access to `cargo`, `crates.io`, and the entire Rust ecosystem.
//! A ΓΛΩΣΣΑ program is just a Rust program with a Greek syntax.
//!
//! ## 3. Performance
//! `rustc` is an incredibly sophisticated optimizing compiler. We don't need to write
//! our own optimization passes (constant folding, loop unrolling, etc.) because `rustc`
//! does it for us.
//!
//! # The Mapping Strategy
//!
//! The mapping between ΓΛΩΣΣΑ and Rust is direct and intentional.
//!
//! | ΓΛΩΣΣΑ Concept | Rust Equivalent | Notes |
//! |----------------|-----------------|-------|
//! | **Types** | | |
//! | `ἀριθμός` (Number) | `i64` | 64-bit signed integer |
//! | `ὄνομα` (String) | `String` | Heap-allocated UTF-8 string |
//! | `ἀληθές/ψεῦδος` | `bool` | Boolean |
//! | `λίστη` | `Vec<T>` | Dynamic array |
//! | `σύνολον` | `HashSet<T>` | Unique collection |
//! | `χάρτης` | `HashMap<K, V>` | Key-value store |
//! | **Control Flow** | | |
//! | `εἰ ...` (If) | `if ...` | Standard control flow |
//! | `εἰ δὲ μή ...` | `else ...` | Else block |
//! | `ἕως ...` (While) | `while ...` | While loop |
//! | `διὰ ...` (For) | `for ... in ...` | Iterator loop |
//! | `παῦε` (Break) | `break` | Break loop |
//! | `συνέχιζε` (Continue)| `continue` | Continue loop |
//! | **Structures** | | |
//! | `εἶδος` (Struct) | `struct` | Generates `#[derive(Clone, Debug, ...)]` |
//! | `χαρακτήρ` (Trait) | `trait` | Maps methods directly |
//! | `ὁρίζειν` (Define) | `impl` | Implementation block |
//! | **Error Handling** | | |
//! | `ἀποτέλεσμα` | `Result<T, E>` | Uses standard Rust Result |
//! | `εὑρεθείη` | `Option<T>` | The "Optative" mood |
//! | `;` (Propagate) | `?` | The Try operator |
//! | `!` (Unwrap) | `.unwrap()` | Panic on failure |
//!
//! # Example Translation
//!
//! **ΓΛΩΣΣΑ Source:**
//! ```text
//! // Define a struct
//! εἶδος Point ὁρίζειν {
//!     x ἀριθμοῦ.
//!     y ἀριθμοῦ.
//! }.
//!
//! // Check equality
//! ξ πέντε ἔστω.
//! εἰ ξ πέντε ἰσοῦται,
//!     «ἰσχύει» λέγε.
//! ```
//!
//! **Generated Rust:**
//! ```rust,ignore
//! #[derive(Clone, Debug, PartialEq)]
//! struct Point {
//!     x: i64,
//!     y: i64,
//! }
//!
//! fn main() {
//!     let x = 5;
//!     if x == 5 {
//!         println!("ἰσχύει");
//!     }
//! }
//! ```
//!
//! # Architecture
//!
//! The codegen process uses the [`quote`] crate to construct a Rust AST (TokenStream)
//! directly from the Semantic Analysis model. This ensures that the generated code
//! is syntactically valid Rust and avoids "stringly typed" code generation errors.
//!
//! 1. **Semantic Analysis**: Produces an [`AnalyzedProgram`](crate::semantic::AnalyzedProgram).
//! 2. **Code Generation**: [`generate_rust`] takes the program and produces a Rust source string.

#![allow(clippy::needless_doctest_main)]

use crate::morphology::lexicon::{BinaryOp, UnaryOp};
use crate::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedMethod, AnalyzedProgram, AnalyzedStatement,
    CaptureMode, GlossaType,
};
use crate::text::normalize_greek;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

// ==================================================================================
// UTILS
// ==================================================================================

/// Capitalize the first letter of a string (for Rust type/trait names)
pub(crate) fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Sanitize a Greek name for use as a Rust identifier
///
/// This function performs the critical step of converting Ancient Greek identifiers
/// into valid ASCII Rust identifiers. It uses a combination of name mapping
/// (for single letters like `α`) and transliteration (for words like `χρήστης`).
///
/// # Edge Cases
///
/// Characters that do not have a standard Latin mapping (like `ϟ` Koppa) are
/// hex-encoded to ensure uniqueness and prevent collisions.
///
/// * `ϟ` -> `_u3df_`
///
/// # Examples
///
/// ```
/// // These are internal functions, but here is how they behave:
/// // sanitize_name("ξ") -> "xi"
/// // sanitize_name("χρήστης") -> "chrestes"
/// ```
pub(crate) fn sanitize_name(name: &str) -> String {
    // Directly transliterate without special casing single letters
    // This prevents collisions between single letters and their full names
    // e.g. "σ" (sigma) vs "σίγμα" (sigma)
    // Prefix with "g_" to namespace all user-defined identifiers and avoid collisions with Rust keywords
    format!("g_{}", transliterate(name))
}

/// Transliterate Greek to Latin characters
pub(crate) fn transliterate(greek: &str) -> String {
    let mut result = String::new();

    for c in greek.chars() {
        let trans = match c {
            'α' => "a",
            'β' => "b",
            'γ' => "g",
            'δ' => "d",
            'ε' => "e",
            'ζ' => "z",
            'η' => "h", // Distinct from 'e' (epsilon)
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
            'ω' => "w", // Distinct from 'o' (omicron)
            // Digraphs and other characters are hex-encoded to prevent collisions
            // θ, φ, χ, ψ map to _u..._ because th, ph, ch, ps collide with sequences
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
    if result.is_empty() {
        return "_var_empty".to_string();
    }

    if result
        .chars()
        .next()
        .map(|c| c.is_numeric())
        .unwrap_or(false)
    {
        format!("_{}", result)
    } else {
        result
    }
}

// ==================================================================================
// TYPES
// ==================================================================================

/// Convert a Glossa type to its Rust equivalent string
///
/// This function recursively traverses complex types (like `Vec<Option<i64>>`)
/// and produces a string that is valid Rust syntax.
///
/// # Examples
///
/// ```
/// use glossa::codegen::to_rust_type;
/// use glossa::semantic::GlossaType;
///
/// // Simple types
/// assert_eq!(to_rust_type(&GlossaType::Number), "i64");
/// assert_eq!(to_rust_type(&GlossaType::String), "String");
///
/// // Complex nested types
/// let list_of_numbers = GlossaType::List(Box::new(GlossaType::Number));
/// assert_eq!(to_rust_type(&list_of_numbers), "Vec<i64>");
///
/// // Result types
/// let result_type = GlossaType::Result(
///     Box::new(GlossaType::Number),
///     Box::new(GlossaType::String)
/// );
/// assert_eq!(to_rust_type(&result_type), "Result<i64, String>");
/// ```
pub fn to_rust_type(ty: &GlossaType) -> String {
    match ty {
        GlossaType::Number => "i64".to_string(),
        GlossaType::String => "String".to_string(),
        GlossaType::Boolean => "bool".to_string(),
        GlossaType::List(inner) => format!("Vec<{}>", to_rust_type(inner)),
        GlossaType::Set(inner) => format!("HashSet<{}>", to_rust_type(inner)),
        GlossaType::Map(key, value) => {
            format!("HashMap<{}, {}>", to_rust_type(key), to_rust_type(value))
        }
        GlossaType::Option(inner) => format!("Option<{}>", to_rust_type(inner)),
        GlossaType::Result(ok, err) => {
            format!("Result<{}, {}>", to_rust_type(ok), to_rust_type(err))
        }
        GlossaType::Unit => "()".to_string(),
        GlossaType::Struct { name, .. } => capitalize(&sanitize_name(name)),
        // TODO: Better representation for function types if they appear in type signatures
        GlossaType::Function { .. } => "fn".to_string(),
        GlossaType::Unknown => "_".to_string(),
    }
}

// ==================================================================================
// RUST CODEGEN
// ==================================================================================

/// Generate Rust code from an Analyzed Program
pub fn generate_rust(program: &AnalyzedProgram) -> String {
    // Check if we need collection imports
    let needs_collections = uses_collections(program);

    // Separate trait defs, struct defs, trait impls, function defs, tests, and main body statements
    let mut trait_defs = Vec::new();
    let mut struct_defs = Vec::new();
    let mut trait_impls = Vec::new();
    let mut fn_defs = Vec::new();
    let mut test_defs = Vec::new();
    let mut main_stmts = Vec::new();

    for stmt in &program.statements {
        match stmt {
            AnalyzedStatement::TraitDefinition { .. } => trait_defs.push(generate_statement(stmt)),
            AnalyzedStatement::TypeDefinition { .. } => struct_defs.push(generate_statement(stmt)),
            AnalyzedStatement::TraitImplementation { .. } => {
                trait_impls.push(generate_statement(stmt))
            }
            AnalyzedStatement::FunctionDef { .. } => fn_defs.push(generate_statement(stmt)),
            AnalyzedStatement::TestDeclaration { .. } => test_defs.push(generate_statement(stmt)),
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

        #(#test_defs)*
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

/// Check if an expression uses collection types
fn expr_uses_collections(expr: &AnalyzedExpr) -> bool {
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

/// Generate a complete Rust file with proper formatting
pub fn generate_rust_file(program: &AnalyzedProgram) -> String {
    let code = generate_rust(program);

    // Add a comment header
    format!(
        "// Generated by ΓΛΩΣΣΑ compiler\n// Αὐτόματος κῶδιξ ἀπὸ ΓΛΩΣΣΑ\n#![allow(non_snake_case, unused_imports)]\n\n{}",
        code
    )
}

/// Generate Rust code for a single analyzed statement
pub fn generate_statement_code(stmt: &AnalyzedStatement) -> String {
    generate_statement(stmt).to_string()
}

fn generate_statement(stmt: &AnalyzedStatement) -> TokenStream {
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

fn generate_expr(expr: &AnalyzedExpr) -> TokenStream {
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

/// Check if a Glossa type maps to a standard Rust type (Vec, String, etc.)
fn is_std_type(ty: &GlossaType) -> bool {
    matches!(
        ty,
        GlossaType::Number
            | GlossaType::String
            | GlossaType::Boolean
            | GlossaType::List(_)
            | GlossaType::Set(_)
            | GlossaType::Map(_, _)
            | GlossaType::Option(_)
            | GlossaType::Result(_, _)
            | GlossaType::Unit
            | GlossaType::Unknown
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;
    use smol_str::SmolStr;

    // --- Utils Tests ---

    #[test]
    fn test_sanitize_greek_letter() {
        assert_eq!(sanitize_name("ξ"), "g_x");
        assert_eq!(sanitize_name("α"), "g_a");
        assert_eq!(sanitize_name("ω"), "g_w");
    }

    #[test]
    fn test_transliterate() {
        // χ (chi) -> hex, ρ -> r, η -> h, σ -> s, τ -> t, ο -> o, ς -> s
        // χ is 0x3c7
        assert_eq!(transliterate("χρηστος"), "_u3c7_rhstos");
        assert_eq!(transliterate("λογος"), "logos");
        // φ (phi) -> hex
        // φ is 0x3c6
        assert_eq!(transliterate("φιλοσοφια"), "_u3c6_iloso_u3c6_ia");
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
    fn test_capitalize() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize("Hello"), "Hello");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("x"), "X");
    }

    #[test]
    fn test_sanitize_keywords_and_prefix() {
        // Test that keywords are safe (by prefixing)
        // If "if" stays "if", it's invalid Rust
        assert_eq!(sanitize_name("if"), "g_if");
        assert_eq!(sanitize_name("fn"), "g_fn");

        // Test that regular identifiers are prefixed
        // This ensures a unique namespace for user variables
        assert_eq!(sanitize_name("x"), "g_x");
        assert_eq!(sanitize_name("foo"), "g_foo");
    }

    // --- Types Tests ---

    #[test]
    fn test_basic_types() {
        assert_eq!(to_rust_type(&GlossaType::Number), "i64");
        assert_eq!(to_rust_type(&GlossaType::String), "String");
        assert_eq!(to_rust_type(&GlossaType::Boolean), "bool");
        assert_eq!(to_rust_type(&GlossaType::Unit), "()");
        assert_eq!(to_rust_type(&GlossaType::Unknown), "_");
    }

    #[test]
    fn test_container_types() {
        assert_eq!(
            to_rust_type(&GlossaType::List(Box::new(GlossaType::Number))),
            "Vec<i64>"
        );
        assert_eq!(
            to_rust_type(&GlossaType::Set(Box::new(GlossaType::String))),
            "HashSet<String>"
        );
        assert_eq!(
            to_rust_type(&GlossaType::Map(
                Box::new(GlossaType::String),
                Box::new(GlossaType::Number)
            )),
            "HashMap<String, i64>"
        );
        assert_eq!(
            to_rust_type(&GlossaType::Option(Box::new(GlossaType::Number))),
            "Option<i64>"
        );
        assert_eq!(
            to_rust_type(&GlossaType::Result(
                Box::new(GlossaType::Number),
                Box::new(GlossaType::String)
            )),
            "Result<i64, String>"
        );
    }

    #[test]
    fn test_struct_type() {
        // Use normalized name as per compiler convention
        let ty = GlossaType::Struct {
            name: SmolStr::new("χρηστης"),
            gender: crate::morphology::Gender::Masculine,
            fields: vec![],
        };
        // Sanitize: χρηστης -> g__u3c7_rhsths
        // Capitalize: g__u3c7_rhsths -> G__u3c7_rhsths
        assert_eq!(to_rust_type(&ty), "G__u3c7_rhsths");
    }

    // --- Rust Codegen Tests ---

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
        // Variables are now prefixed with g_
        assert!(code.contains("let g_x"));
        assert!(code.contains("5"));
    }

    #[test]
    fn test_generate_number() {
        let code = compile("42 λέγε.");
        assert!(code.contains("println"), "Expected println in: {}", code);
        assert!(code.contains("42"));
    }

    #[test]
    fn test_generate_full_program() {
        let code = compile("ξ πέντε ἔστω. ξ λέγε.");
        // Variables are now prefixed with g_
        assert!(
            code.contains("let g_x = 5"),
            "Expected binding in: {}",
            code
        );
        assert!(code.contains("println"), "Expected println in: {}", code);
    }

    #[test]
    fn test_generate_statement_code() {
        let ast = parse("«χαῖρε» λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let stmt = &analyzed.statements[0];
        let code = generate_statement_code(stmt);
        assert!(code.contains("println"));
        assert!(code.contains("χαῖρε"));
    }

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
