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
//! The codegen process uses the [`mod@quote`] crate to construct a Rust AST (TokenStream)
//! directly from the Semantic Analysis model. This ensures that the generated code
//! is syntactically valid Rust and avoids "stringly typed" code generation errors.
//!
//! 1. **Semantic Analysis**: Produces an [`AnalyzedProgram`].
//! 2. **Code Generation**: [`generate_rust`] takes the program and produces a Rust source string.

#![allow(clippy::needless_doctest_main)]

use crate::morphology::lexicon::{BinaryOp, UnaryOp};
use crate::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedMethod, AnalyzedProgram, AnalyzedStatement,
    CaptureMode, GlossaType,
};
use crate::text::normalize_greek;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
// ==================================================================================
// UTILS
// ==================================================================================

/// Helper to sanitize a name and format it as an Ident
pub(crate) fn sanitize_ident(name: &str) -> Ident {
    format_ident!(
        "{}",
        Sanitizer {
            name,
            capitalize: false
        }
        .to_string()
    )
}

/// Zero-allocation sanitizer for Rust identifiers
struct Sanitizer<'a> {
    name: &'a str,
    capitalize: bool,
}

impl<'a> std::fmt::Display for Sanitizer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.capitalize {
            f.write_str("G_")?;
        } else {
            f.write_str("g_")?;
        }

        transliterate_fmt(self.name, f)?;

        if self.name.is_empty() {
            f.write_str("_var_empty")?;
        }

        Ok(())
    }
}

/// Sanitize a Greek name for use as a Rust identifier
///
/// This function performs the critical step of converting Ancient Greek identifiers
/// into valid ASCII Rust identifiers. It exclusively uses hex-encoding for all
/// non-ASCII characters to guarantee uniqueness and prevent collisions.
///
/// # The Strategy: Hex Encoding
///
/// Instead of trying to map Greek letters to Latin letters (which is lossy and
/// prone to collisions like `χ` -> `ch` vs `c` + `h`), we simply hex-encode
/// the Unicode scalar value of every non-ASCII character.
///
/// * `α` (U+03B1) -> `_u3b1_`
/// * `ξ` (U+03BE) -> `_u3be_`
///
/// This ensures that `x` (ASCII) and `ξ` (Greek Xi) are distinct in the generated Rust code.
///
/// # Examples
///
/// ```rust
/// use glossa::codegen::sanitize_name;
///
/// // Hex encoding (prefixed with g_ for namespace safety)
/// assert_eq!(sanitize_name("ξ"), "g__u3be_");
/// assert_eq!(sanitize_name("χρηστης"), "g__u3c7__u3c1__u3b7__u3c3__u3c4__u3b7__u3c2_");
///
/// // Keyword safety (even Rust keywords are safe due to g_ prefix)
/// assert_eq!(sanitize_name("if"), "g_if");
/// ```
pub fn sanitize_name(name: &str) -> String {
    // Use the zero-allocation Sanitizer to generate the string
    Sanitizer {
        name,
        capitalize: false,
    }
    .to_string()
}

/// Transliterate Greek to Latin characters via Hex Encoding
///
/// This function maps Greek characters (and any non-ASCII character) to a
/// hex-encoded sequence `_uXXXX_`. It ensures that the output contains only
/// valid Rust identifier characters (alphanumeric + underscore).
///
/// **Note:** This function expects normalized (monotonic) Greek text, but will
/// work correctly (by hex-encoding) on any input.
///
/// # Mapping Strategy
///
/// * **ASCII Alphanumeric + `_`**: Kept as-is.
/// * **Everything else**: Hex-encoded as `_uXXXX_`.
///
/// This strategy is "lossless" for identifiers and guarantees no collisions.
///
/// # Examples
///
/// ```rust
/// use glossa::codegen::transliterate;
///
/// assert_eq!(transliterate("λογος"), "_u3bb__u3bf__u3b3__u3bf__u3c2_");
/// assert_eq!(transliterate("φιλοσοφια"), "_u3c6__u3b9__u3bb__u3bf__u3c3__u3bf__u3c6__u3b9__u3b1_");
/// ```
pub fn transliterate(greek: &str) -> String {
    if greek.is_empty() {
        return "_var_empty".to_string();
    }

    // Optimization: Check if we need to prepend '_' if result would start with a digit.
    // Since only ASCII alphanumeric pass through as-is, and hex-encoded chars start with '_',
    // the only case where result starts with digit is if input starts with ASCII digit.
    let starts_numeric = greek.starts_with(|c: char| c.is_ascii_digit());

    let mut result = String::with_capacity(greek.len() * 2 + if starts_numeric { 1 } else { 0 });

    if starts_numeric {
        result.push('_');
    }

    // We can unwrap safely because writing to String never fails
    transliterate_fmt(greek, &mut result).unwrap();

    result
}

/// Helper to transliterate directly into a generic writer
#[inline]
fn transliterate_fmt<W: std::fmt::Write>(text: &str, result: &mut W) -> std::fmt::Result {
    for c in text.chars() {
        // We now hex-encode ALL Greek characters to avoid collisions with ASCII.
        // Previously, 'ξ' mapped to 'x', causing collision with ASCII 'x'.
        // Now, 'ξ' maps to '_u3be_', which is distinct from 'x' (which stays 'x').
        if c.is_ascii_alphanumeric() || c == '_' {
            result.write_char(c)?;
        } else {
            // Replace invalid characters with unique hex code to prevent collisions
            write!(result, "_u{:x}_", c as u32)?;
        }
    }
    Ok(())
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
use std::fmt::Write;

pub fn to_rust_type(ty: &GlossaType) -> String {
    let mut result = String::with_capacity(32);
    write_rust_type(ty, &mut result).unwrap();
    result
}

fn write_rust_type(ty: &GlossaType, out: &mut String) -> std::fmt::Result {
    match ty {
        GlossaType::Number => write!(out, "i64"),
        GlossaType::String => write!(out, "String"),
        GlossaType::Boolean => write!(out, "bool"),
        GlossaType::List(inner) => {
            write!(out, "Vec<")?;
            write_rust_type(inner, out)?;
            write!(out, ">")
        }
        GlossaType::Set(inner) => {
            write!(out, "HashSet<")?;
            write_rust_type(inner, out)?;
            write!(out, ">")
        }
        GlossaType::Map(key, value) => {
            write!(out, "HashMap<")?;
            write_rust_type(key, out)?;
            write!(out, ", ")?;
            write_rust_type(value, out)?;
            write!(out, ">")
        }
        GlossaType::Option(inner) => {
            write!(out, "Option<")?;
            write_rust_type(inner, out)?;
            write!(out, ">")
        }
        GlossaType::Result(ok, err) => {
            write!(out, "Result<")?;
            write_rust_type(ok, out)?;
            write!(out, ", ")?;
            write_rust_type(err, out)?;
            write!(out, ">")
        }
        GlossaType::Unit => write!(out, "()"),
        GlossaType::Struct { name, .. } => {
            let sanitized = Sanitizer {
                name,
                capitalize: true,
            }
            .to_string();
            write!(out, "{}", sanitized)
        }
        // TODO: Better representation for function types if they appear in type signatures
        GlossaType::Function { .. } => write!(out, "fn"),
        GlossaType::Unknown => write!(out, "_"),
    }
}

/// Generates the Rust token stream for a given `GlossaType`.
///
/// # Examples
/// ```
/// use glossa::semantic::GlossaType;
/// use glossa::codegen::generate_type_tokens;
/// let tokens = generate_type_tokens(&GlossaType::Number);
/// assert_eq!(tokens.to_string(), "i64");
/// ```
pub fn generate_type_tokens(ty: &GlossaType) -> TokenStream {
    match ty {
        GlossaType::Number => quote! { i64 },
        GlossaType::String => quote! { String },
        GlossaType::Boolean => quote! { bool },
        GlossaType::List(inner) => {
            let inner_tokens = generate_type_tokens(inner);
            quote! { Vec<#inner_tokens> }
        }
        GlossaType::Set(inner) => {
            let inner_tokens = generate_type_tokens(inner);
            quote! { HashSet<#inner_tokens> }
        }
        GlossaType::Map(key, value) => {
            let key_tokens = generate_type_tokens(key);
            let value_tokens = generate_type_tokens(value);
            quote! { HashMap<#key_tokens, #value_tokens> }
        }
        GlossaType::Option(inner) => {
            let inner_tokens = generate_type_tokens(inner);
            quote! { Option<#inner_tokens> }
        }
        GlossaType::Result(ok, err) => {
            let ok_tokens = generate_type_tokens(ok);
            let err_tokens = generate_type_tokens(err);
            quote! { Result<#ok_tokens, #err_tokens> }
        }
        GlossaType::Unit => quote! { () },
        GlossaType::Struct { name, .. } => {
            // Directly sanitize and create identifier without intermediate string allocation for the type name
            let ident = format_ident!(
                "{}",
                Sanitizer {
                    name,
                    capitalize: true
                }
                .to_string()
            );
            quote! { #ident }
        }
        // Function types are not fully supported in type signatures yet, fallback to generic
        GlossaType::Function { .. } => quote! { fn },
        GlossaType::Unknown => quote! { _ },
    }
}

// ==================================================================================
// RUST CODEGEN
// ==================================================================================

/// Translates an [`AnalyzedProgram`] directly into a raw Rust source code string.
///
/// # Examples
/// ```
/// use glossa::parser::parse;
/// use glossa::semantic::analyze_program;
/// use glossa::codegen::generate_rust;
///
/// let ast = parse("«χαῖρε» λέγε.").unwrap();
/// let program = analyze_program(&ast).unwrap();
/// let rust_code = generate_rust(&program);
/// assert!(rust_code.contains("println"));
/// ```
pub fn generate_rust(program: &AnalyzedProgram) -> String {
    // Separate trait defs, struct defs, trait impls, function defs, tests, and main body statements
    // ⚡ Bolt Optimization: Pre-allocate vectors based on statement length.
    // This reduces internal reallocation overhead during partitioning.
    let capacity = program.statements.len();
    let mut trait_defs = Vec::with_capacity(capacity);
    let mut struct_defs = Vec::with_capacity(capacity);
    let mut trait_impls = Vec::with_capacity(capacity);
    let mut fn_defs = Vec::with_capacity(capacity);
    let mut test_defs = Vec::with_capacity(capacity);
    let mut main_stmts = Vec::with_capacity(capacity);

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

    // Generate imports
    // We always import collections because they might be used in type definitions
    // even if not explicitly constructed in the code.
    // Unused imports are suppressed by #![allow(unused_imports)] in the file header.
    let imports = quote! { use std::collections::{HashMap, HashSet}; };

    let panic_hook = generate_panic_hook();

    let output = quote! {
        #imports

        #(#trait_defs)*

        #(#struct_defs)*

        #(#trait_impls)*

        #(#fn_defs)*

        fn main() {
            #panic_hook

            #(#main_stmts)*
        }

        #(#test_defs)*
    };

    output.to_string()
}

fn generate_panic_hook() -> TokenStream {
    quote! {
        std::panic::set_hook(Box::new(|info| {
            let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
                *s
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                &s[..]
            } else {
                "Unknown error"
            };

            let (greek_msg, english_msg) = match msg {
                "division by zero" | "division by zero or overflow" =>
                    ("Διαίρεσις διὰ μηδενός", "Division by zero"),
                "arithmetic overflow" | "attempt to add with overflow" | "attempt to subtract with overflow" | "attempt to multiply with overflow" =>
                    ("Ὑπερχείλισις ἀριθμοῦ", "Arithmetic overflow"),
                "index out of bounds" | "bounds check" =>
                    ("Δείκτης ἐκτὸς ὁρίων", "Index out of bounds"),
                s if s.starts_with("index out of bounds") =>
                    ("Δείκτης ἐκτὸς ὁρίων", "Index out of bounds"),
                _ => ("Σφάλμα ἐκτελέσεως", msg),
            };

            eprintln!("✕ {}: {}", greek_msg, english_msg);

            if let Some(location) = info.location() {
                eprintln!("  (at line {})", location.line());
            }
        }));
    }
}

/// Transpiles a ΓΛΩΣΣΑ program into a complete, standalone Rust file (`main.rs`).
///
/// # Examples
/// ```
/// use glossa::parser::parse;
/// use glossa::semantic::analyze_program;
/// use glossa::codegen::generate_rust_file;
///
/// let ast = parse("«χαῖρε» λέγε.").unwrap();
/// let program = analyze_program(&ast).unwrap();
/// let rust_file = generate_rust_file(&program);
/// assert!(rust_file.contains("fn main"));
/// ```
pub fn generate_rust_file(program: &AnalyzedProgram) -> String {
    let code = generate_rust(program);

    // Add a comment header
    format!(
        "// Generated by ΓΛΩΣΣΑ compiler\n// Αὐτόματος κῶδιξ ἀπὸ ΓΛΩΣΣΑ\n#![deny(unsafe_code)]\n#![allow(non_snake_case, unused_imports)]\nuse std::convert::TryFrom;\n\n{}",
        code
    )
}

/// Generate Rust code for a single analyzed statement
///
/// # Why it exists
///
/// This function acts as a bridge between the AST semantic analysis phase and the Rust source
/// text generation. While most of the internal code generation relies heavily on the `TokenStream`
/// structure provided by the `quote` crate (which handles things like syntactic validation and
/// hygiene internally), it is sometimes necessary to obtain a raw `String` representation of a
/// translated statement, such as when embedding fragments or writing out to a file.
///
/// ## Examples
///
/// ```rust
/// use glossa::semantic::{AnalyzedStatement, AnalyzedExpr, AnalyzedExprKind, GlossaType};
/// use smol_str::SmolStr;
/// use glossa::codegen::generate_statement_code;
///
/// let stmt = AnalyzedStatement::Binding {
///     name: SmolStr::new("ξ"),
///     value: AnalyzedExpr {
///         expr: AnalyzedExprKind::NumberLiteral(5),
///         glossa_type: GlossaType::Number,
///     },
///     mutable: false,
/// };
///
/// let rust_code = generate_statement_code(&stmt);
/// // The resulting code string is `let g__u3be_ = 5i64 ;`
/// ```
pub fn generate_statement_code(stmt: &AnalyzedStatement) -> String {
    generate_statement(stmt).to_string()
}

fn generate_statements(stmts: &[AnalyzedStatement]) -> Vec<TokenStream> {
    stmts.iter().map(generate_statement).collect()
}

fn generate_statement(stmt: &AnalyzedStatement) -> TokenStream {
    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => generate_statement_binding(name, value, *mutable),
        AnalyzedStatement::Assignment { name, value } => generate_statement_assignment(name, value),
        AnalyzedStatement::Print(exprs) | AnalyzedStatement::Query(exprs) => generate_print(exprs),
        AnalyzedStatement::Expression(exprs) => generate_statement_expression(exprs),
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
        AnalyzedStatement::Return { value } => generate_statement_return(value),
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

fn generate_statement_binding(name: &str, value: &AnalyzedExpr, mutable: bool) -> TokenStream {
    // Check if it's an array to force mutable
    let is_array = matches!(value.expr, AnalyzedExprKind::ArrayLiteral(_));
    generate_let(name, value, mutable || is_array)
}

fn generate_statement_assignment(name: &str, value: &AnalyzedExpr) -> TokenStream {
    let name_ident = sanitize_ident(name);
    let value_tokens = generate_expr(value);
    quote! { #name_ident = #value_tokens; }
}

fn generate_statement_expression(exprs: &[AnalyzedExpr]) -> TokenStream {
    // ⚡ Bolt Optimization: Removed intermediate `.collect::<Vec<_>>()` allocation.
    let expr_tokens = exprs.iter().map(|e| {
        let tokens = generate_expr(e);
        quote! { #tokens; }
    });
    quote! { #(#expr_tokens)* }
}

fn generate_statement_return(value: &Option<Box<AnalyzedExpr>>) -> TokenStream {
    match value {
        Some(e) => {
            let value_tokens = generate_expr(e);
            quote! { return #value_tokens; }
        }
        None => quote! { return; },
    }
}

fn generate_let(name: &str, value: &AnalyzedExpr, mutable: bool) -> TokenStream {
    let name_ident = sanitize_ident(name);
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
    } else {
        // ⚡ Bolt Optimization: Build the format string directly to avoid the O(n) heap
        // allocation of the intermediate Vec<_> created by .collect::<Vec<_>>().join(" ").
        let mut format_str = String::with_capacity(args.len() * 3 - 1);
        for i in 0..args.len() {
            if i > 0 {
                format_str.push(' ');
            }
            format_str.push_str("{}");
        }

        // ⚡ Bolt Optimization: Removed intermediate `.collect::<Vec<_>>()` allocation.
        let arg_tokens = args.iter().map(generate_expr);
        quote! { println!(#format_str, #(#arg_tokens),*); }
    }
}

fn generate_if(
    condition: &AnalyzedExpr,
    then_body: &[AnalyzedStatement],
    else_body: &Option<Vec<AnalyzedStatement>>,
) -> TokenStream {
    let cond = generate_expr(condition);
    let then_stmts = generate_statements(then_body);

    match else_body {
        Some(else_stmts) => {
            let else_tokens = generate_statements(else_stmts);
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
    let body_stmts = generate_statements(body);
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
    let var_ident = sanitize_ident(variable);
    let iter = generate_expr(iterator);
    let body_stmts = generate_statements(body);
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
    let arm_tokens = arms.iter().map(|(pattern, body)| {
        // Check if pattern is wildcard (represented as BooleanLiteral(true))
        let pat = if matches!(pattern.expr, AnalyzedExprKind::BooleanLiteral(true)) {
            // Generate wildcard pattern
            quote! { _ }
        } else {
            generate_expr(pattern)
        };
        let body_stmts = generate_statements(body);
        quote! { #pat => { #(#body_stmts)* } }
    });

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
    let fn_name = sanitize_ident(name);
    let body_stmts = generate_statements(body);

    // Generate parameter list
    let param_tokens = params.iter().map(|(param_name, param_type)| {
        let param_ident = sanitize_ident(param_name);
        if let Some(ty) = param_type {
            let ty_tokens = generate_type_tokens(ty);
            quote! { #param_ident: #ty_tokens }
        } else {
            quote! { #param_ident }
        }
    });

    // Generate return type
    if let Some(ret_type) = return_type {
        let ret_tokens = generate_type_tokens(ret_type);
        quote! {
            fn #fn_name(#(#param_tokens),*) -> #ret_tokens {
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
    let struct_name = format_ident!(
        "{}",
        Sanitizer {
            name,
            capitalize: true
        }
        .to_string()
    );

    // Generate field list
    let field_tokens = fields.iter().map(|(field_name, field_type)| {
        let field_ident = sanitize_ident(field_name);
        let type_tokens = generate_type_tokens(field_type);
        quote! { #field_ident: #type_tokens }
    });

    quote! {
        #[derive(Debug, Clone)]
        struct #struct_name {
            #(#field_tokens),*
        }
    }
}

struct TraitMethodParts {
    name: Ident,
    params: Vec<TokenStream>,
    return_type: Option<TokenStream>,
}

fn generate_trait_method_parts(method: &AnalyzedMethod) -> TraitMethodParts {
    let method_name = sanitize_ident(&method.name);

    let param_tokens = method
        .params
        .iter()
        .enumerate()
        .map(|(idx, (param_name, param_type))| {
            if is_self_parameter(param_name, idx) {
                quote! { &self }
            } else {
                let param_ident = sanitize_ident(param_name);
                let type_tokens = generate_type_tokens(param_type);
                quote! { #param_ident: #type_tokens }
            }
        });

    let ret_tokens = method.return_type.as_ref().map(generate_type_tokens);

    TraitMethodParts {
        name: method_name,
        params: param_tokens.collect(),
        return_type: ret_tokens,
    }
}

fn generate_trait_def(name: &str, methods: &[AnalyzedMethod]) -> TokenStream {
    // Capitalize trait name for Rust conventions
    let trait_name = format_ident!(
        "{}",
        Sanitizer {
            name,
            capitalize: true
        }
        .to_string()
    );

    // Generate method signatures
    let method_tokens = methods.iter().map(|method| {
        let parts = generate_trait_method_parts(method);
        let method_name = parts.name;
        let param_tokens = parts.params;

        if let Some(ret_ty) = parts.return_type {
            if let Some(body) = &method.body {
                let body_stmts = generate_statements(body);
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
            let body_stmts = generate_statements(body);
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
    });

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
    let trait_ident = format_ident!(
        "{}",
        Sanitizer {
            name: trait_name,
            capitalize: true
        }
        .to_string()
    );
    let type_ident = format_ident!(
        "{}",
        Sanitizer {
            name: type_name,
            capitalize: true
        }
        .to_string()
    );

    // Generate method implementations
    let method_tokens = methods.iter().map(|method| {
        let parts = generate_trait_method_parts(method);
        let method_name = parts.name;
        let param_tokens = parts.params;

        // Generate method body
        let body_stmts: Vec<TokenStream> = if let Some(body) = &method.body {
            generate_statements(body)
        } else {
            Vec::new()
        };

        if let Some(ret_ty) = parts.return_type {
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
    });

    quote! {
        impl #trait_ident for #type_ident {
            #(#method_tokens)*
        }
    }
}

fn generate_test(name: &str, body: &[AnalyzedStatement]) -> TokenStream {
    // Sanitize test name for Rust function identifier
    // Replace spaces and special chars with underscores
    let mut test_fn_name = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_alphanumeric() {
            for lc in c.to_lowercase() {
                test_fn_name.push(lc);
            }
        } else if c == ' ' || c == '-' || c == '_' {
            test_fn_name.push('_');
        }
    }

    let test_ident = format_ident!("test_{}", test_fn_name);

    // Generate body statements
    let body_stmts = generate_statements(body);

    quote! {
        #[test]
        fn #test_ident() {
            #(#body_stmts)*
        }
    }
}

fn generate_expr_some(inner: &AnalyzedExpr) -> TokenStream {
    let inner_tokens = generate_expr(inner);
    quote! { Some(#inner_tokens) }
}

fn generate_expr_ok(inner: &AnalyzedExpr) -> TokenStream {
    let inner_tokens = generate_expr(inner);
    quote! { Ok(#inner_tokens) }
}

fn generate_expr_err(inner: &AnalyzedExpr) -> TokenStream {
    let inner_tokens = generate_expr(inner);
    quote! { Err(#inner_tokens) }
}

fn generate_expr_try(inner: &AnalyzedExpr) -> TokenStream {
    let inner_tokens = generate_expr(inner);
    quote! { #inner_tokens? }
}

fn generate_expr_unwrap(inner: &AnalyzedExpr) -> TokenStream {
    let inner_tokens = generate_expr(inner);
    quote! { #inner_tokens.expect("attempted to unwrap an empty value") }
}

fn generate_expr(expr: &AnalyzedExpr) -> TokenStream {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => generate_literal_string(s),

        AnalyzedExprKind::NumberLiteral(n) => generate_literal_number(*n),

        AnalyzedExprKind::BooleanLiteral(b) => generate_literal_boolean(*b),

        AnalyzedExprKind::ArrayLiteral(elements) => generate_collection_array(elements),

        AnalyzedExprKind::Some(inner) => generate_expr_some(inner),

        AnalyzedExprKind::None => quote! { None },

        AnalyzedExprKind::Ok(inner) => generate_expr_ok(inner),

        AnalyzedExprKind::Err(inner) => generate_expr_err(inner),

        AnalyzedExprKind::Try(inner) => generate_expr_try(inner),

        AnalyzedExprKind::Unwrap(inner) => generate_expr_unwrap(inner),

        AnalyzedExprKind::IndexAccess { array, index } => generate_collection_index(array, index),

        AnalyzedExprKind::Variable(name) => generate_variable(name),

        AnalyzedExprKind::PropertyAccess { owner, property } => {
            generate_property_access(owner, property)
        }

        AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } => generate_method_call(receiver, method, args),

        AnalyzedExprKind::VerbCall { verb, args }
        | AnalyzedExprKind::FunctionCall { func: verb, args } => generate_function_call(verb, args),

        AnalyzedExprKind::BinOp { op, left, right } => generate_bin_op(*op, left, right),

        AnalyzedExprKind::UnaryOp { op, operand } => generate_unary_op(*op, operand),

        AnalyzedExprKind::Range {
            start,
            end,
            inclusive,
        } => generate_range(start, end, *inclusive),

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
            generate_collection_new(collection_type)
        }

        AnalyzedExprKind::Assert { condition } => generate_control_assert(condition),

        AnalyzedExprKind::AssertEq { left, right } => generate_control_assert_eq(left, right),
    }
}

fn generate_bin_op(op: BinaryOp, left: &AnalyzedExpr, right: &AnalyzedExpr) -> TokenStream {
    let left_tokens = generate_expr(left);
    let right_tokens = generate_expr(right);

    // Use checked arithmetic only for numeric types
    let use_checked = matches!(left.glossa_type, GlossaType::Number);

    if use_checked {
        match op {
            BinaryOp::Add => {
                return generate_checked_op(
                    left_tokens,
                    right_tokens,
                    "checked_add",
                    "arithmetic overflow",
                );
            }
            BinaryOp::Sub => {
                return generate_checked_op(
                    left_tokens,
                    right_tokens,
                    "checked_sub",
                    "arithmetic overflow",
                );
            }
            BinaryOp::Mul => {
                return generate_checked_op(
                    left_tokens,
                    right_tokens,
                    "checked_mul",
                    "arithmetic overflow",
                );
            }
            BinaryOp::Div => {
                return generate_checked_op(
                    left_tokens,
                    right_tokens,
                    "checked_div",
                    "division by zero or overflow",
                );
            }
            BinaryOp::Mod => {
                return generate_checked_op(
                    left_tokens,
                    right_tokens,
                    "checked_rem",
                    "division by zero or overflow",
                );
            }
            _ => {} // Fallthrough to standard logic for comparisons
        }
    }

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

fn generate_checked_op(
    left: TokenStream,
    right: TokenStream,
    method: &str,
    msg: &str,
) -> TokenStream {
    let method_ident = format_ident!("{}", method);
    quote! { (#left).#method_ident(#right).expect(#msg) }
}

fn generate_struct_lit(
    type_name: &str,
    fields: &[smol_str::SmolStr],
    args: &[AnalyzedExpr],
) -> TokenStream {
    // Capitalize struct name for Rust conventions
    let struct_name = format_ident!(
        "{}",
        Sanitizer {
            name: type_name,
            capitalize: true
        }
        .to_string()
    );

    // Generate field: value pairs using actual field names
    let field_assignments = fields.iter().zip(args.iter()).map(|(field_name, arg)| {
        let field_ident = sanitize_ident(field_name);
        let arg_token = generate_expr(arg);
        quote! { #field_ident: #arg_token }
    });

    quote! { #struct_name { #(#field_assignments),* } }
}

fn generate_closure(
    params: &[smol_str::SmolStr],
    body: &AnalyzedExpr,
    capture_mode: &CaptureMode,
) -> TokenStream {
    let body_tokens = generate_expr(body);
    let params_idents: Vec<_> = params.iter().map(|p| sanitize_ident(p)).collect();

    match capture_mode {
        CaptureMode::Move => {
            quote! { move |#(#params_idents),*| #body_tokens }
        }
        CaptureMode::Borrow => {
            quote! { |#(#params_idents),*| #body_tokens }
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
            | "pop"
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

// ==================================================================================
// EXPRESSION HELPERS (SIMPLE)
// ==================================================================================

fn generate_literal_string(s: &str) -> TokenStream {
    quote! { #s }
}

fn generate_literal_number(n: i64) -> TokenStream {
    quote! { #n }
}

fn generate_literal_boolean(b: bool) -> TokenStream {
    quote! { #b }
}

fn generate_variable(name: &str) -> TokenStream {
    let name_ident = sanitize_ident(name);
    quote! { #name_ident }
}

fn generate_property_access(owner: &AnalyzedExpr, property: &str) -> TokenStream {
    let obj = generate_expr(owner);
    let field_ident = sanitize_ident(property);
    quote! { #obj.#field_ident }
}

fn generate_unary_op(op: UnaryOp, operand: &AnalyzedExpr) -> TokenStream {
    match op {
        UnaryOp::Not => {
            let operand_tokens = generate_expr(operand);
            quote! { !#operand_tokens }
        }
        UnaryOp::Neg => {
            let operand_tokens = generate_expr(operand);
            if matches!(operand.glossa_type, GlossaType::Number) {
                quote! { (#operand_tokens).checked_neg().expect("arithmetic overflow") }
            } else {
                quote! { -#operand_tokens }
            }
        }
        UnaryOp::Ref => {
            let operand_tokens = generate_expr(operand);
            quote! { &#operand_tokens }
        }
    }
}

fn generate_range(start: &AnalyzedExpr, end: &AnalyzedExpr, inclusive: bool) -> TokenStream {
    let start_tokens = generate_expr(start);
    let end_tokens = generate_expr(end);
    if inclusive {
        quote! { (#start_tokens..=#end_tokens) }
    } else {
        quote! { (#start_tokens..#end_tokens) }
    }
}

// ==================================================================================
// EXPRESSION HELPERS (COMPLEX)
// ==================================================================================

fn generate_collection_array(elements: &[AnalyzedExpr]) -> TokenStream {
    // ⚡ Bolt Optimization: Removed intermediate `.collect::<Vec<_>>()` allocation.
    let elem_tokens = elements.iter().map(generate_expr);
    quote! { vec![#(#elem_tokens),*] }
}

fn generate_collection_new(collection_type: &str) -> TokenStream {
    // Generate HashSet::new() or HashMap::new()
    let type_ident = format_ident!("{}", collection_type);
    quote! { #type_ident::new() }
}

fn generate_collection_index(array: &AnalyzedExpr, index: &AnalyzedExpr) -> TokenStream {
    let array_tokens = generate_expr(array);
    let index_tokens = generate_expr(index);
    // Safety check for negative index
    quote! {
        {
            let idx = #index_tokens;
            if idx < 0 {
                panic!("index out of bounds: negative index {}", idx);
            }
            let u_idx = usize::try_from(idx).expect("index out of bounds: too large");
            #array_tokens.get(u_idx).cloned().expect("index out of bounds: index too large")
        }
    }
}

fn generate_method_call(
    receiver: &AnalyzedExpr,
    method: &str,
    args: &[AnalyzedExpr],
) -> TokenStream {
    let recv = generate_expr(receiver);

    // Check if this is a standard library method call on a standard type
    let method_ident = if is_std_method(method) && is_std_type(&receiver.glossa_type) {
        // Use the raw method name (e.g., "len", "push")
        format_ident!("{}", method)
    } else {
        // Sanitize (prefix with g_) for user-defined methods
        sanitize_ident(method)
    };

    // ⚡ Bolt Optimization: Removed intermediate `.collect::<Vec<_>>()` allocation.
    let arg_tokens = args.iter().map(generate_expr);
    quote! { #recv.#method_ident(#(#arg_tokens),*) }
}

fn generate_function_call(verb: &str, args: &[AnalyzedExpr]) -> TokenStream {
    let func_ident = sanitize_ident(verb);
    // ⚡ Bolt Optimization: Removed intermediate `.collect::<Vec<_>>()` allocation.
    let arg_tokens = args.iter().map(generate_expr);
    quote! { #func_ident(#(#arg_tokens),*) }
}

// ==================================================================================
// EXPRESSION HELPERS (CONTROL FLOW)
// ==================================================================================

fn generate_control_assert(condition: &AnalyzedExpr) -> TokenStream {
    let cond = generate_expr(condition);
    quote! { assert!(#cond) }
}

fn generate_control_assert_eq(left: &AnalyzedExpr, right: &AnalyzedExpr) -> TokenStream {
    let left_tokens = generate_expr(left);
    let right_tokens = generate_expr(right);
    quote! { assert_eq!(#left_tokens, #right_tokens) }
}

#[cfg(test)]
#[cfg(test)]
mod tests;
