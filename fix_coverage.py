import re

with open("src/semantic/control_flow.rs", "r") as f:
    content = f.read()

# Simplify the word_opt logic to remove dead code
old_code = """                    let word_opt = if let Expr::Word(w) = e {
                        Some(w)
                    } else if let Expr::Phrase(terms) = e {
                        if terms.len() == 1 {
                            if let Expr::Word(w) = &terms[0] {
                                Some(w)
                            } else { None }
                        } else { None }
                    } else { None };"""

new_code = """                    let word_opt = if let Expr::Word(w) = e {
                        Some(w)
                    } else { None };"""

content = content.replace(old_code, new_code)

with open("src/semantic/control_flow.rs", "w") as f:
    f.write(content)
