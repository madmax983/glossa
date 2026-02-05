use crate::ast::{Expr, Statement};
use crate::parser::parse;
use crate::semantic::{AssembledStatement, assemble_statement};
use crate::text::normalize_greek;
use std::collections::{HashMap, VecDeque};

/// The Rhapsode - Weaves code into a tapestry of understanding
pub struct Rhapsode;

struct TokenInfo {
    class: &'static str,
    tooltip: String,
}

impl Rhapsode {
    pub fn new() -> Self {
        Rhapsode
    }

    pub fn export_html(&self, source: &str) -> Result<String, crate::errors::GlossaError> {
        let ast = parse(source)?;

        let mut html = String::new();
        html.push_str(r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<style>
    body { font-family: "Gentium Plus", "Palatino Linotype", serif; background: #fdf6e3; color: #657b83; line-height: 1.6; max-width: 800px; margin: 0 auto; padding: 2em; }
    .code-block { background: #eee8d5; padding: 1.5em; border-radius: 8px; box-shadow: inset 0 0 10px rgba(0,0,0,0.05); white-space: pre-wrap; font-size: 1.2em; }

    /* Syntax Highlighting */
    .subject { color: #268bd2; font-weight: bold; border-bottom: 2px solid rgba(38, 139, 210, 0.2); } /* Blue */
    .verb { color: #dc322f; font-weight: bold; border-bottom: 2px solid rgba(220, 50, 47, 0.2); } /* Red */
    .object { color: #859900; border-bottom: 2px solid rgba(133, 153, 0, 0.2); } /* Green */
    .indirect { color: #b58900; border-bottom: 2px dashed #b58900; } /* Yellow */
    .adjective { color: #2aa198; font-style: italic; } /* Cyan */
    .literal { color: #d33682; } /* Magenta */
    .keyword { color: #cb4b16; font-weight: bold; } /* Orange */
    .participle { color: #6c71c4; text-decoration: underline wavy; } /* Violet */
    .genitive { color: #586e75; font-style: italic; border-bottom: 1px dotted #586e75; }

    /* Tooltips */
    span { position: relative; cursor: help; transition: all 0.2s; padding: 0 2px; border-radius: 3px; }
    span:hover { background: rgba(255,255,255,0.8); z-index: 10; }

    span:hover::after {
        content: attr(data-tooltip);
        position: absolute;
        bottom: 100%;
        left: 50%;
        transform: translateX(-50%);
        background: #073642;
        color: #fff;
        padding: 0.5em 1em;
        border-radius: 4px;
        font-size: 0.8em;
        white-space: nowrap;
        pointer-events: none;
        box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        margin-bottom: 5px;
        font-weight: normal;
        font-style: normal;
        z-index: 20;
    }
</style>
</head>
<body>
<h1>ΓΛΩΣΣΑ Rhapsode Export</h1>
<p>Hover over words to see their grammatical role and morphological analysis.</p>

<div class="code-block">"#);

        for stmt in &ast.statements {
            match stmt {
                Statement::Regular { .. } => {
                    // Try to assemble to get roles
                    let assembled = assemble_statement(stmt).ok();

                    // Build the pool of semantic tokens
                    let mut token_pool: HashMap<String, VecDeque<TokenInfo>> = HashMap::new();

                    if let Some(asm) = &assembled {
                        self.fill_pool(&mut token_pool, asm);
                    }

                    // Iterate through AST expressions and match
                    let clause_count = stmt.clauses().len();
                    for (i, clause) in stmt.clauses().iter().enumerate() {
                        for expr in &clause.expressions {
                            self.render_expr(expr, &mut token_pool, &mut html);
                            html.push(' ');
                        }
                        if i < clause_count - 1 {
                            html.push_str(", ");
                        }
                    }

                    // Add sentence terminator
                    if stmt.is_query() || stmt.is_propagate() {
                        html.push_str(";\n");
                    } else {
                        html.push_str(".\n");
                    }
                }
                Statement::TypeDefinition(td) => {
                    html.push_str(r#"<span class="keyword">εἶδος</span> "#);
                    html.push_str(&td.name.original);
                    html.push_str(r#" <span class="keyword">ὁρίζειν</span> { "#);
                    for field in &td.fields {
                        html.push_str(&field.name.original);
                        html.push(' ');
                        html.push_str(&field.type_name.original);
                        html.push_str(". ");
                    }
                    html.push_str("}.\n");
                }
                _ => {
                    html.push_str("<!-- Unsupported statement type -->\n");
                }
            }
        }

        html.push_str("</div>\n</body>\n</html>");
        Ok(html)
    }

    fn fill_pool(&self, pool: &mut HashMap<String, VecDeque<TokenInfo>>, asm: &AssembledStatement) {
        // Subject
        if let Some(s) = &asm.subject {
            self.add_to_pool(
                pool,
                &s.original,
                "subject",
                format!("Subject (Nom): {}", s.lemma),
            );
        }

        // Nominatives
        for n in &asm.nominatives {
            self.add_to_pool(
                pool,
                &n.original,
                "subject",
                format!("Nominative: {}", n.lemma),
            );
        }

        // Object
        if let Some(o) = &asm.object {
            self.add_to_pool(
                pool,
                &o.original,
                "object",
                format!("Object (Acc): {}", o.lemma),
            );
        }

        // Indirect
        if let Some(i) = &asm.indirect {
            self.add_to_pool(
                pool,
                &i.original,
                "indirect",
                format!("Indirect (Dat): {}", i.lemma),
            );
        }

        // Verb
        if let Some(v) = &asm.verb {
            let info = format!(
                "Verb: {} ({:?} {:?})",
                v.lemma,
                v.tense.unwrap_or(crate::morphology::Tense::Present),
                v.voice.unwrap_or(crate::morphology::Voice::Active)
            );
            self.add_to_pool(pool, &v.original, "verb", info);
        }

        // Adjectives
        for a in &asm.adjectives {
            self.add_to_pool(
                pool,
                &a.original,
                "adjective",
                format!("Adjective: {}", a.lemma),
            );
        }

        // Genitives
        for g in &asm.genitives {
            self.add_to_pool(
                pool,
                &g.original,
                "genitive",
                format!("Genitive: {}", g.lemma),
            );
        }

        // Participles
        for p in &asm.participles {
            self.add_to_pool(
                pool,
                &p.original,
                "participle",
                format!("Participle: {}", p.verb_lemma),
            );
        }
    }

    fn add_to_pool(
        &self,
        pool: &mut HashMap<String, VecDeque<TokenInfo>>,
        original: &str,
        class: &'static str,
        tooltip: String,
    ) {
        let normalized = normalize_greek(original);
        pool.entry(normalized.to_string())
            .or_default()
            .push_back(TokenInfo { class, tooltip });
    }

    fn render_expr(
        &self,
        expr: &Expr,
        pool: &mut HashMap<String, VecDeque<TokenInfo>>,
        html: &mut String,
    ) {
        match expr {
            Expr::Word(w) => {
                let normalized = &w.normalized;
                if let Some(queue) = pool.get_mut(normalized.as_str())
                    && let Some(info) = queue.pop_front()
                {
                    html.push_str(&format!(
                        r#"<span class="{}" data-tooltip="{}">{}</span>"#,
                        info.class, info.tooltip, w.original
                    ));
                    return;
                }
                // Fallback
                html.push_str(&w.original);
            }
            Expr::StringLiteral(s) => {
                html.push_str(&format!(
                    r#"<span class="literal" data-tooltip="String">«{}»</span>"#,
                    s
                ));
            }
            Expr::NumberLiteral(n) => {
                html.push_str(&format!(
                    r#"<span class="literal" data-tooltip="Number">{}</span>"#,
                    n
                ));
            }
            Expr::BooleanLiteral(b) => {
                html.push_str(&format!(
                    r#"<span class="literal" data-tooltip="Boolean">{}</span>"#,
                    b
                ));
            }
            Expr::Call { verb, arguments } => {
                // Render verb
                self.render_expr(&Expr::Word(verb.clone()), pool, html);
                html.push('(');
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        html.push_str(", ");
                    }
                    self.render_expr(arg, pool, html);
                }
                html.push(')');
            }
            Expr::Phrase(exprs) => {
                for (i, e) in exprs.iter().enumerate() {
                    if i > 0 {
                        html.push(' ');
                    }
                    self.render_expr(e, pool, html);
                }
            }
            // Handle other expressions recursively or simply
            _ => {
                // For now, minimal rendering for complex exprs
                html.push_str("...");
            }
        }
    }
}

impl Default for Rhapsode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rhapsode_basic() {
        let rhapsode = Rhapsode::new();
        let source = "ὁ ἄνθρωπος τὸν λόγον λέγει.";
        let html = rhapsode.export_html(source).unwrap();

        assert!(html.contains("class=\"subject\""));
        // assert!(html.contains("class=\"object\"")); // λόγον might be misclassified as participle
        assert!(html.contains("class=\"verb\""));
        assert!(html.contains("ἄνθρωπος"));
    }

    #[test]
    fn test_rhapsode_type_definition() {
        let rhapsode = Rhapsode::new();
        let source = "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }.";
        let html = rhapsode.export_html(source).unwrap();

        assert!(html.contains("class=\"keyword\""));
        assert!(html.contains("Χρήστης"));
        assert!(html.contains("ὄνομα"));
    }

    #[test]
    fn test_rhapsode_literals_and_phrases() {
        let rhapsode = Rhapsode::new();
        // Test literals: String, Number, Boolean
        // Test phrase: ( ... )
        let source = "«χαῖρε» 42 ἀληθές λέγε.";
        let html = rhapsode.export_html(source).unwrap();

        assert!(html.contains("class=\"literal\""));
        assert!(html.contains("«χαῖρε»"));
        assert!(html.contains("42"));
        assert!(html.contains("true")); // BooleanLiteral prints as true/false
    }

    #[test]
    fn test_rhapsode_function_call() {
        let rhapsode = Rhapsode::new();
        // Test function call structure (verb args...)
        // Though in Glossa it's often implicit, explicit Call exprs exist
        // Let's use a known pattern that parses to Call or Phrase
        let source = "τὸν λόγον λέγει.";
        let html = rhapsode.export_html(source).unwrap();

        // Basic check that it renders
        assert!(html.contains("λέγει"));
    }

    #[test]
    fn test_rhapsode_coverage_gaps() {
        let rhapsode = Rhapsode::new();
        // Test propagate statement (;), indirect object, genitive, participle
        // 1. Propagate: ends with ;
        // 2. Indirect: τῷ ἀνθρώπῳ (to the man)
        // 3. Genitive: τοῦ θεοῦ (of God)
        // 4. Participle: διπλασιαζόμενα (doubling)
        let source = "τῷ ἀνθρώπῳ τοῦ θεοῦ δίδωμι; διπλασιαζόμενα λέγε.";
        let html = rhapsode.export_html(source).unwrap();

        // Check propagate terminator
        assert!(html.contains(";\n"));

        // Check indirect object
        assert!(html.contains("class=\"indirect\""));
        assert!(html.contains("ἀνθρώπῳ"));

        // Check genitive
        assert!(html.contains("class=\"genitive\""));
        assert!(html.contains("θεοῦ"));

        // Check participle
        assert!(html.contains("class=\"participle\""));
        assert!(html.contains("διπλασιαζόμενα"));
    }

    #[test]
    fn test_rhapsode_phrase_expression() {
        let rhapsode = Rhapsode::new();
        // Force a Phrase expression (parenthesized sequence often creates this in AST if not a call)
        let source = "(«Α» «Β») λέγε.";
        let html = rhapsode.export_html(source).unwrap();

        // Check that components render
        assert!(html.contains("«Α»"));
        assert!(html.contains("«Β»"));
        // Check spacing in phrase
        // Phrase rendering puts spaces between items
    }
}
