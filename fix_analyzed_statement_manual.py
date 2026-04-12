import re

with open('src/semantic/model.rs', 'r') as f:
    content = f.read()

content = content.replace("            AnalyzedStatement::For { item_name, iterator, variable, body } => AnalyzedStatement::For { item_name: item_name.clone(), iterator: iterator.clone(), variable: variable.clone(), body: body.clone() },\n", "            AnalyzedStatement::For { iterator, variable, body } => AnalyzedStatement::For { iterator: iterator.clone(), variable: variable.clone(), body: body.clone() },\n")

with open('src/semantic/model.rs', 'w') as f:
    f.write(content)

with open('src/semantic/control_flow.rs', 'r') as f:
    cf_content = f.read()
cf_content = cf_content.replace("        AnalyzedStatement::Expression(mut exprs) => {", "        AnalyzedStatement::Expression(ref exprs) => {")
cf_content = cf_content.replace("        AnalyzedStatement::Binding { name, value, .. } => {", "        AnalyzedStatement::Binding { ref name, ref value, .. } => {")
cf_content = cf_content.replace("            return Ok(AnalyzedStatement::Expression(exprs));", "            return Ok(AnalyzedStatement::Expression(exprs.clone()));")
cf_content = cf_content.replace("            return Ok(AnalyzedStatement::Binding {", "            return Ok(AnalyzedStatement::Binding {")
cf_content = cf_content.replace("                name: name,", "                name: name.clone(),")
cf_content = cf_content.replace("                value: value,", "                value: value.clone(),")

with open('src/semantic/control_flow.rs', 'w') as f:
    f.write(cf_content)
