//! Stela - Documentation Generator for ΓΛΩΣΣΑ
//!
//! "Code is ephemeral, but Stelae are eternal."
//!
//! This module generates HTML documentation from ΓΛΩΣΣΑ source files,
//! styling them as ancient inscriptions.

use crate::grammar::{GlossaParser, Rule};
use miette::{IntoDiagnostic, Result};
use pest::Parser;
use std::fs;
use std::path::Path;

/// Generate documentation for a ΓΛΩΣΣΑ file
pub fn generate_docs(input: &Path, output: Option<&Path>) -> Result<()> {
    let source = fs::read_to_string(input).into_diagnostic()?;
    let items = parse_documented_items(&source)?;

    let html = render_html(
        input.file_name().unwrap().to_string_lossy().as_ref(),
        &items,
    );

    if let Some(out_path) = output {
        fs::write(out_path, html).into_diagnostic()?;
        println!("Stela created at: {}", out_path.display());
    } else {
        println!("{}", html);
    }

    Ok(())
}

struct DocItem {
    kind: String,
    name: String,
    signature: String,
    docs: Vec<String>,
}

fn parse_documented_items(source: &str) -> Result<Vec<DocItem>> {
    let mut items = Vec::new();

    // Parse the full program to get statement locations
    // We use the raw parser directly to get Spans
    let pairs = GlossaParser::parse(Rule::program, source)
        .map_err(|e| miette::miette!("Parse error: {}", e))?;

    let mut last_end = 0;

    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for stmt in pair.into_inner() {
                if stmt.as_rule() == Rule::statement {
                    let span = stmt.as_span();
                    let start = span.start();

                    // Extract comments from the gap between last_end and start
                    let gap = &source[last_end..start];
                    let docs = extract_comments(gap);

                    // Identify the statement type and name
                    if let Some(item) = analyze_statement(stmt, docs, source) {
                        items.push(item);
                    }

                    last_end = span.end();
                }
            }
        }
    }

    Ok(items)
}

fn extract_comments(text: &str) -> Vec<String> {
    let mut comments = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(comment) = trimmed.strip_prefix("//") {
            comments.push(comment.trim().to_string());
        }
    }
    comments
}

fn analyze_statement(
    pair: pest::iterators::Pair<Rule>,
    docs: Vec<String>,
    _full_source: &str,
) -> Option<DocItem> {
    // Look inside the statement wrapper
    let inner = pair.into_inner().next()?;
    let rule = inner.as_rule();

    let (kind, name) = match rule {
        Rule::type_definition => {
            let name = extract_name(inner.clone(), Rule::greek_word)?;
            ("Type (Εἶδος)", name)
        }
        Rule::trait_definition => {
            let name = extract_name(inner.clone(), Rule::greek_word)?;
            ("Trait (Χαρακτήρ)", name)
        }
        Rule::trait_impl => {
            // "Type implementing Trait"
            let type_name = extract_name(inner.clone(), Rule::greek_word)?;
            // We need the second greek word for trait name, simplified for now
            ("Implementation", type_name)
        }
        Rule::test_declaration => {
            let name = extract_string_content(inner.clone())?;
            ("Test (Δοκιμή)", name)
        }
        _ => {
            // For regular statements, we only include them if they have docs
            if docs.is_empty() {
                return None;
            }
            ("Statement", "Expression".to_string())
        }
    };

    let signature = inner
        .as_span()
        .as_str()
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    let signature = if signature.len() > 50 {
        format!("{}...", &signature[..47])
    } else {
        signature
    };

    Some(DocItem {
        kind: kind.to_string(),
        name,
        signature,
        docs,
    })
}

fn extract_name(pair: pest::iterators::Pair<Rule>, target_rule: Rule) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == target_rule {
            return Some(inner.as_str().to_string());
        }
    }
    None
}

fn extract_string_content(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::string_literal {
            for content in inner.into_inner() {
                if content.as_rule() == Rule::string_content {
                    return Some(content.as_str().to_string());
                }
            }
        }
    }
    None
}

fn render_html(filename: &str, items: &[DocItem]) -> String {
    let mut body = String::new();

    for item in items {
        let docs_html: String = item
            .docs
            .iter()
            .map(|d| format!("<p class=\"comment\">{}</p>", escape_html(d)))
            .collect();

        let section = format!(
            r#"
            <div class="item">
                <div class="meta">
                    <span class="kind">{}</span>
                    <span class="name">{}</span>
                </div>
                <div class="signature">{}</div>
                <div class="docs">
                    {}
                </div>
            </div>
            "#,
            item.kind,
            escape_html(&item.name),
            escape_html(&item.signature),
            docs_html
        );
        body.push_str(&section);
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="el">
<head>
    <meta charset="UTF-8">
    <title>Stela: {}</title>
    <style>
        @import url('https://fonts.googleapis.com/css2?family=Noto+Serif+Display:ital,wght@0,400;0,700;1,400&family=Noto+Sans:wght@300;400;700&display=swap');

        :root {{
            --bg-color: #fdf6e3;
            --text-color: #3b3b3b;
            --stone-color: #657b83;
            --accent-color: #b58900;
            --border-color: #93a1a1;
        }}

        body {{
            font-family: 'Noto Serif Display', serif;
            background-color: var(--bg-color);
            color: var(--text-color);
            max-width: 800px;
            margin: 0 auto;
            padding: 40px;
            line-height: 1.6;
        }}

        header {{
            text-align: center;
            margin-bottom: 60px;
            border-bottom: 2px solid var(--accent-color);
            padding-bottom: 20px;
        }}

        h1 {{
            font-size: 3em;
            margin: 0;
            color: var(--accent-color);
            text-transform: uppercase;
            letter-spacing: 0.1em;
        }}

        .subtitle {{
            font-family: 'Noto Sans', sans-serif;
            color: var(--stone-color);
            font-size: 1.2em;
            margin-top: 10px;
        }}

        .item {{
            margin-bottom: 40px;
            padding: 20px;
            border-left: 4px solid var(--border-color);
            background: rgba(0,0,0,0.02);
        }}

        .meta {{
            display: flex;
            align-items: baseline;
            gap: 10px;
            margin-bottom: 10px;
        }}

        .kind {{
            font-family: 'Noto Sans', sans-serif;
            font-size: 0.8em;
            text-transform: uppercase;
            color: var(--stone-color);
            font-weight: bold;
        }}

        .name {{
            font-size: 1.5em;
            font-weight: bold;
            color: var(--text-color);
        }}

        .signature {{
            font-family: 'Courier New', monospace;
            background: rgba(255,255,255,0.5);
            padding: 5px 10px;
            border-radius: 4px;
            margin-bottom: 15px;
            font-size: 0.9em;
            color: var(--stone-color);
        }}

        .docs p {{
            margin: 0.5em 0;
        }}

        .footer {{
            margin-top: 60px;
            text-align: center;
            font-size: 0.8em;
            color: var(--stone-color);
            border-top: 1px solid var(--border-color);
            padding-top: 20px;
        }}
    </style>
</head>
<body>
    <header>
        <h1>ΓΛΩΣΣΑ</h1>
        <div class="subtitle">Stela of {}</div>
    </header>

    <main>
        {}
    </main>

    <div class="footer">
        Generated by Glossa Stela
    </div>
</body>
</html>"#,
        escape_html(filename),
        escape_html(filename),
        body
    )
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
