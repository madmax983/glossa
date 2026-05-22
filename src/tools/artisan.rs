//! The Artisan (ὁ Τεχνίτης) - JS Transpiler
//!
//! A CLI tool that converts analyzed Glossa programs to JavaScript code.
//! This allows the Glossa logic to run natively in a browser or Node.js environment.

use std::fmt::Write;
use std::path::Path;
use miette::Result;

use crate::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement,
};
use crate::morphology::{BinaryOp, UnaryOp};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;

/// Run the JS transpiler on a Glossa source file
pub fn run_artisan(input: &Path) -> Result<()> {
    let source = load_source(input)?;
    let status = Status::start_with_symbol("Τεχνίτης (Transpiling to JS)", "🛠️");

    let analyzed = match crate::tools::runner::analyze_source(&source) {
        Ok(a) => a,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    let js_code = transpile_to_js(&analyzed);

    // Generate HTML Wrapper
    let html_content = generate_html_wrapper(&js_code);

    // Write to file adjacent to input
    let out_path = input.with_extension("html");
    if let Err(e) = std::fs::write(&out_path, html_content) {
        status.error("Failed to write HTML");
        return Err(miette::miette!("Failed to write HTML: {}", e));
    }

    // Fix UI issue: success() doesn't take arguments
    status.success();
    println!("Transpiled to {}", out_path.display());

    Ok(())
}

fn generate_html_wrapper(js_code: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ΓΛΩΣΣΑ - JS Artisan</title>
    <style>
        body {{
            font-family: 'Courier New', Courier, monospace;
            background-color: #1e1e1e;
            color: #d4d4d4;
            padding: 2rem;
            max-width: 800px;
            margin: 0 auto;
        }}
        h1 {{
            color: #569cd6;
            text-align: center;
        }}
        #terminal {{
            background-color: #000;
            padding: 1rem;
            border-radius: 8px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
            min-height: 300px;
            overflow-y: auto;
            border: 1px solid #333;
        }}
        .log-entry {{
            margin: 0.2rem 0;
        }}
        .button-container {{
            text-align: center;
            margin-bottom: 1rem;
        }}
        button {{
            background-color: #4CAF50;
            border: none;
            color: white;
            padding: 10px 24px;
            text-align: center;
            text-decoration: none;
            display: inline-block;
            font-size: 16px;
            margin: 4px 2px;
            cursor: pointer;
            border-radius: 4px;
            font-family: inherit;
        }}
        button:hover {{
            background-color: #45a049;
        }}
    </style>
</head>
<body>
    <h1>ΓΛΩΣΣΑ Browser Execution</h1>
    <div class="button-container">
        <button id="runBtn">Run Program</button>
        <button id="clearBtn" style="background-color: #f44336;">Clear Terminal</button>
    </div>
    <div id="terminal"></div>

    <script>
        const terminal = document.getElementById('terminal');

        // Intercept console.log
        const originalLog = console.log;
        console.log = function(...args) {{
            originalLog.apply(console, args);
            const msg = args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' ');
            const div = document.createElement('div');
            div.className = 'log-entry';
            div.textContent = '> ' + msg;
            terminal.appendChild(div);
            terminal.scrollTop = terminal.scrollHeight;
        }};

        document.getElementById('clearBtn').addEventListener('click', () => {{
            terminal.innerHTML = '';
        }});

        document.getElementById('runBtn').addEventListener('click', () => {{
            console.log("--- Starting Execution ---");
            try {{
                // User Code execution block
                (() => {{
{}
                }})();
            }} catch (e) {{
                console.log("Error: " + e.message);
            }}
            console.log("--- Execution Finished ---");
        }});
    </script>
</body>
</html>"#,
        js_code
    )
}

fn transpile_to_js(program: &AnalyzedProgram) -> String {
    let mut out = String::with_capacity(1024);
    for stmt in &program.statements {
        out.push_str(&transpile_statement(stmt, 5)); // 5 indents for HTML wrapper alignment
        out.push('\n');
    }
    out
}

fn transpile_statement(stmt: &AnalyzedStatement, indent: usize) -> String {
    let ind = "    ".repeat(indent);
    match stmt {
        AnalyzedStatement::Print(exprs) => {
            let args = format_transpiled_exprs(exprs);
            format!("{}console.log({});", ind, args)
        }
        AnalyzedStatement::Binding { name, value, mutable, .. } => {
            let keyword = if *mutable { "let" } else { "const" };
            format!("{}// Variable declaration\n{}{} {} = {};", ind, ind, keyword, sanitize_ident(name), transpile_expr(value))
        }
        AnalyzedStatement::Assignment { name, value, .. } => {
            format!("{}{} = {};", ind, sanitize_ident(name), transpile_expr(value))
        }
        AnalyzedStatement::Expression(exprs) => {
            let mut out = String::new();
            for expr in exprs {
                out.push_str(&format!("{}{};\n", ind, transpile_expr(expr)));
            }
            out.trim_end().to_string()
        }
        AnalyzedStatement::If { condition, then_body, else_body } => {
            let mut out = format!("{}if ({}) {{\n", ind, transpile_expr(condition));
            for b_stmt in then_body {
                out.push_str(&transpile_statement(b_stmt, indent + 1));
                out.push('\n');
            }
            out.push_str(&format!("{}}}", ind));

            if let Some(else_branch) = else_body {
                out.push_str(" else {\n");
                for b_stmt in else_branch {
                    out.push_str(&transpile_statement(b_stmt, indent + 1));
                    out.push('\n');
                }
                out.push_str(&format!("{}}}", ind));
            }
            out
        }
        AnalyzedStatement::While { condition, body } => {
            let mut out = format!("{}while ({}) {{\n", ind, transpile_expr(condition));
            for b_stmt in body {
                out.push_str(&transpile_statement(b_stmt, indent + 1));
                out.push('\n');
            }
            out.push_str(&format!("{}}}", ind));
            out
        }
        AnalyzedStatement::FunctionDef { name, params, body, .. } => {
            let params_str = params.iter().map(|(n, _)| sanitize_ident(n)).collect::<Vec<_>>().join(", ");
            let safe_name = sanitize_ident(name);
            let mut out = format!("{}function {}({}) {{\n", ind, safe_name, params_str);

            for (i, b_stmt) in body.iter().enumerate() {
                let mut is_last_expr = false;
                if i == body.len() - 1 && matches!(b_stmt, AnalyzedStatement::Expression(_)) {
                    is_last_expr = true;
                }

                if is_last_expr {
                    if let AnalyzedStatement::Expression(exprs) = b_stmt {
                        for (j, expr) in exprs.iter().enumerate() {
                            if j == exprs.len() - 1 {
                                out.push_str(&format!("{}    return {};\n", ind, transpile_expr(expr)));
                            } else {
                                out.push_str(&format!("{}    {};\n", ind, transpile_expr(expr)));
                            }
                        }
                    }
                } else {
                    out.push_str(&transpile_statement(b_stmt, indent + 1));
                    out.push('\n');
                }
            }
            out.push_str(&format!("{}}}", ind));
            out
        }
        AnalyzedStatement::Return { value } => {
            if let Some(v) = value {
                format!("{}return {};", ind, transpile_expr(v))
            } else {
                format!("{}return;", ind)
            }
        }
        AnalyzedStatement::TypeDefinition { name, fields, .. } => {
            let mut out = format!("{}class {} {{\n{}    constructor({{", ind, sanitize_ident(name), ind);
            let field_names = fields.iter().map(|(n, _)| sanitize_ident(n)).collect::<Vec<_>>().join(", ");
            out.push_str(&field_names);
            out.push_str("}) {\n");
            for (f_name, _) in fields {
                let s_name = sanitize_ident(f_name);
                out.push_str(&format!("{}        this.{} = {};\n", ind, s_name, s_name));
            }
            out.push_str(&format!("{}    }}\n{}}}", ind, ind));
            out
        }
        AnalyzedStatement::TestDeclaration { .. } |
        AnalyzedStatement::TraitDefinition { .. } |
        AnalyzedStatement::TraitImplementation { .. } => {
            format!("{}/* Untranspiled declaration */", ind)
        }
        AnalyzedStatement::Match { scrutinee, .. } => {
            // Using JS switch for now as a simple approximation if it fits,
            // but for full pattern matching we'd need if/else chains.
            // We'll output a comment for unsupported advanced matches.
            format!("{}/* Match statement on {} not fully supported in simple JS transpilation yet */", ind, transpile_expr(scrutinee))
        }
        // These don't exist, we will use a fallback catch-all
        _ => {
            format!("{}/* Untranspiled statement */", ind)
        }
    }
}

fn transpile_expr(expr: &AnalyzedExpr) -> String {
    match &expr.expr {
        AnalyzedExprKind::NumberLiteral(n) => n.to_string(),
        AnalyzedExprKind::StringLiteral(s) => format!("\"{}\"", s),
        AnalyzedExprKind::BooleanLiteral(b) => (if *b { "true" } else { "false" }).to_string(),
        AnalyzedExprKind::Variable(name) => sanitize_ident(name),
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            format!("{}.{}", transpile_expr(owner), sanitize_ident(property))
        }
        AnalyzedExprKind::VerbCall { verb, args } => {
            format!("{}({})", sanitize_ident(verb), format_transpiled_exprs(args))
        }
        AnalyzedExprKind::FunctionCall { func, args } => {
            format!("{}({})", sanitize_ident(func), format_transpiled_exprs(args))
        }
        AnalyzedExprKind::StructInstantiation { type_name, fields, args } => {
            let mut kw_args_buf = String::with_capacity(fields.len() * 16);
            for (i, (f, a)) in fields.iter().zip(args.iter()).enumerate() {
                if i > 0 {
                    kw_args_buf.push_str(", ");
                }
                let _ = write!(&mut kw_args_buf, "{}: {}", sanitize_ident(f), transpile_expr(a));
            }
            format!("new {}({{{}}})", sanitize_ident(type_name), kw_args_buf)
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => {
            format!("[{}]", format_transpiled_exprs(exprs))
        }
        AnalyzedExprKind::BinOp { left, op, right } => {
            let l = transpile_expr(left);
            let r = transpile_expr(right);
            let op_str = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Mod => "%",
                BinaryOp::Eq => "===",
                BinaryOp::Ne => "!==",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                BinaryOp::And => "&&",
                BinaryOp::Or => "||",
            };
            format!("({} {} {})", l, op_str, r)
        }
        AnalyzedExprKind::UnaryOp { op, operand } => {
            let o = transpile_expr(operand);
            match op {
                UnaryOp::Neg => format!("-{}", o),
                UnaryOp::Not => format!("!{}", o),
                UnaryOp::Ref => o, // JS doesn't have explicit refs
            }
        }
        AnalyzedExprKind::Lambda { params, body, .. } => {
             let params_str = params.iter().map(|n| sanitize_ident(n)).collect::<Vec<_>>().join(", ");
             format!("({}) => {}", params_str, transpile_expr(body))
        }
        AnalyzedExprKind::MethodCall { receiver, method, args } => {
            format!("{}.{}({})", transpile_expr(receiver), sanitize_ident(method), format_transpiled_exprs(args))
        }
        AnalyzedExprKind::IndexAccess { array, index } => {
            format!("{}[{}]", transpile_expr(array), transpile_expr(index))
        }
        _ => "/* Unimplemented expr */".to_string(),
    }
}

fn format_transpiled_exprs(exprs: &[AnalyzedExpr]) -> String {
    exprs.iter().map(transpile_expr).collect::<Vec<_>>().join(", ")
}

fn sanitize_ident(name: &str) -> String {
    let safe_name = name.replace(" ", "_").replace("-", "_").replace("\"", "");
    format!("g_{}", safe_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    fn transpile_code(code: &str) -> String {
        let ast = parse(code).unwrap();
        let program = analyze_program(&ast).unwrap();
        transpile_to_js(&program)
    }

    #[test]
    fn test_transpile_print() {
        let code = "«χαῖρε κόσμε» λέγε.";
        let js = transpile_code(code);
        assert!(js.contains("console.log(\"χαῖρε κόσμε\")"));
    }

    #[test]
    fn test_transpile_variables() {
        let code = "ξ πέντε ἔστω. ξ λέγε.";
        let js = transpile_code(code);
        assert!(js.contains("const g_ξ = 5"));
        assert!(js.contains("console.log(g_ξ)"));
    }

    #[test]
    fn test_transpile_arithmetic() {
        let code = "ξ 1 2 ἄθροισμα ἔστω.";
        let js = transpile_code(code);
        assert!(js.contains("g_ξ = (1 + 2)"));
    }

    #[test]
    fn test_transpile_function() {
        let code = "πρόσθεσις ὁρίζειν τῷ α ἀριθμοῦ τῷ β ἀριθμοῦ · α β ἄθροισμα δός.";
        let js = transpile_code(code);
        assert!(js.contains("function g_προσθεσις(g_α, g_β)"));
        assert!(js.contains("return (g_α + g_β)"));
    }

    #[test]
    fn test_html_wrapper() {
        let html = generate_html_wrapper("console.log(1);");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("console.log(1);"));
        assert!(html.contains("<div id=\"terminal\"></div>"));
    }
}
