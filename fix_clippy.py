with open('src/semantic/assembly/mod.rs', 'r') as f:
    content = f.read()

content = content.replace("    is_match_arm: bool,", "")
content = content.replace("            is_match_arm: false,", "")
content = content.replace("""        if let Some(subj) = &self.state.subject {
            if subj.lemma.chars().count() == 1 {
                return Ok(());
            }
        }""", """        if let Some(subj) = &self.state.subject {
            if subj.lemma.chars().count() == 1 {
                return Ok(());
            }
        }""") # Wait, clippy suggested using `&&` but the let chain is not fully stable, wait, let chains are stable now in 1.94.

with open('src/semantic/assembly/mod.rs', 'w') as f:
    f.write(content)

with open('src/semantic/control_flow.rs', 'r') as f:
    content = f.read()

# Let's just fix it using clippy's auto-fix
