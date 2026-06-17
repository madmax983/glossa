import re

with open("src/tools/narrator.rs", "r") as f:
    content = f.read()

def replace_all(text, replacements):
    for old, new in replacements.items():
        text = text.replace(old, new)
    return text

replacements = {
    # Replace format_exprs definition
    """fn format_exprs(exprs: &[AnalyzedExpr]) -> String {
    let mut buf = String::with_capacity(exprs.len() * 16);
    for (i, expr) in exprs.iter().enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        buf.push_str(&tell_expr(expr));
    }
    buf
}""": """fn format_exprs(exprs: &[AnalyzedExpr], buf: &mut String) {
    for (i, expr) in exprs.iter().enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        tell_expr_into(expr, buf);
    }
}""",

    # Replace format_types definition
    """fn format_types(types: &[GlossaType]) -> String {
    let mut buf = String::with_capacity(types.len() * 16);
    for (i, ty) in types.iter().enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        buf.push_str(&tell_type(ty));
    }
    buf
}""": """fn format_types(types: &[GlossaType], buf: &mut String) {
    for (i, ty) in types.iter().enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        tell_type_into(ty, buf);
    }
}""",

    # tell_expr -> String and tell_expr_into(&mut String)
    """pub(crate) fn tell_expr(expr: &AnalyzedExpr) -> String {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => format!("\\"{}\\"", s),
        AnalyzedExprKind::NumberLiteral(n) => format!("{}", n),
        AnalyzedExprKind::BooleanLiteral(b) => format!("{}", b),
        AnalyzedExprKind::Variable(name) => format!("`{}`", name),
        AnalyzedExprKind::VerbCall { verb, args } => tell_verb_call(verb, args),
        AnalyzedExprKind::BinOp { left, op, right } => {
            format!("({} {:?} {})", tell_expr(left), op, tell_expr(right))
        }
        AnalyzedExprKind::UnaryOp { op, operand } => {
            format!("({:?} {})", op, tell_expr(operand))
        }
        AnalyzedExprKind::Range {
            start,
            end,
            inclusive,
        } => {
            let range_op = if *inclusive { "..=" } else { ".." };
            format!("{}{}{}", tell_expr(start), range_op, tell_expr(end))
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => tell_array_literal(exprs),
        AnalyzedExprKind::Some(e) => format!("Some({})", tell_expr(e)),
        AnalyzedExprKind::None => "None".to_string(),
        AnalyzedExprKind::Ok(e) => format!("Ok({})", tell_expr(e)),
        AnalyzedExprKind::Err(e) => format!("Err({})", tell_expr(e)),
        AnalyzedExprKind::Unwrap(e) => format!("{}!", tell_expr(e)),
        AnalyzedExprKind::Try(e) => format!("{}?", tell_expr(e)),
        AnalyzedExprKind::IndexAccess { array, index } => {
            format!("{}[{}]", tell_expr(array), tell_expr(index))
        }
        AnalyzedExprKind::FunctionCall { func, args } => tell_function_call(func, args),
        AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } => tell_method_call(receiver, method, args),
        AnalyzedExprKind::StructInstantiation {
            type_name,
            fields,
            args,
        } => tell_struct_instantiation(type_name, fields, args),
        AnalyzedExprKind::Lambda {
            params,
            body,
            capture_mode,
        } => tell_lambda(params, body, capture_mode),
        AnalyzedExprKind::CollectionNew { collection_type } => {
            format!("{}::new()", collection_type)
        }
        AnalyzedExprKind::Assert { condition } => {
            format!("assert({})", tell_expr(condition))
        }
        AnalyzedExprKind::AssertEq { left, right } => {
            format!("assert_eq({}, {})", tell_expr(left), tell_expr(right))
        }
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            format!("{}.{}", tell_expr(owner), property)
        }
    }
}""": """#[allow(dead_code)]
pub(crate) fn tell_expr(expr: &AnalyzedExpr) -> String {
    let mut buf = String::with_capacity(32);
    tell_expr_into(expr, &mut buf);
    buf
}

pub(crate) fn tell_expr_into(expr: &AnalyzedExpr, buf: &mut String) {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => { let _ = write!(buf, "\\"{}\\"", s); },
        AnalyzedExprKind::NumberLiteral(n) => { let _ = write!(buf, "{}", n); },
        AnalyzedExprKind::BooleanLiteral(b) => { let _ = write!(buf, "{}", b); },
        AnalyzedExprKind::Variable(name) => { let _ = write!(buf, "`{}`", name); },
        AnalyzedExprKind::VerbCall { verb, args } => tell_verb_call_into(verb, args, buf),
        AnalyzedExprKind::BinOp { left, op, right } => {
            buf.push('(');
            tell_expr_into(left, buf);
            let _ = write!(buf, " {:?} ", op);
            tell_expr_into(right, buf);
            buf.push(')');
        }
        AnalyzedExprKind::UnaryOp { op, operand } => {
            let _ = write!(buf, "({:?} ", op);
            tell_expr_into(operand, buf);
            buf.push(')');
        }
        AnalyzedExprKind::Range {
            start,
            end,
            inclusive,
        } => {
            tell_expr_into(start, buf);
            let range_op = if *inclusive { "..=" } else { ".." };
            buf.push_str(range_op);
            tell_expr_into(end, buf);
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => tell_array_literal_into(exprs, buf),
        AnalyzedExprKind::Some(e) => {
            buf.push_str("Some(");
            tell_expr_into(e, buf);
            buf.push(')');
        }
        AnalyzedExprKind::None => buf.push_str("None"),
        AnalyzedExprKind::Ok(e) => {
            buf.push_str("Ok(");
            tell_expr_into(e, buf);
            buf.push(')');
        }
        AnalyzedExprKind::Err(e) => {
            buf.push_str("Err(");
            tell_expr_into(e, buf);
            buf.push(')');
        }
        AnalyzedExprKind::Unwrap(e) => {
            tell_expr_into(e, buf);
            buf.push('!');
        }
        AnalyzedExprKind::Try(e) => {
            tell_expr_into(e, buf);
            buf.push('?');
        }
        AnalyzedExprKind::IndexAccess { array, index } => {
            tell_expr_into(array, buf);
            buf.push('[');
            tell_expr_into(index, buf);
            buf.push(']');
        }
        AnalyzedExprKind::FunctionCall { func, args } => tell_function_call_into(func, args, buf),
        AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } => tell_method_call_into(receiver, method, args, buf),
        AnalyzedExprKind::StructInstantiation {
            type_name,
            fields,
            args,
        } => tell_struct_instantiation_into(type_name, fields, args, buf),
        AnalyzedExprKind::Lambda {
            params,
            body,
            capture_mode,
        } => tell_lambda_into(params, body, capture_mode, buf),
        AnalyzedExprKind::CollectionNew { collection_type } => {
            let _ = write!(buf, "{}::new()", collection_type);
        }
        AnalyzedExprKind::Assert { condition } => {
            buf.push_str("assert(");
            tell_expr_into(condition, buf);
            buf.push(')');
        }
        AnalyzedExprKind::AssertEq { left, right } => {
            buf.push_str("assert_eq(");
            tell_expr_into(left, buf);
            buf.push_str(", ");
            tell_expr_into(right, buf);
            buf.push(')');
        }
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            tell_expr_into(owner, buf);
            let _ = write!(buf, ".{}", property);
        }
    }
}""",

    # Helper functions
    """fn tell_verb_call(verb: &str, args: &[AnalyzedExpr]) -> String {
    format!("{}({})", verb, format_exprs(args))
}

fn tell_array_literal(exprs: &[AnalyzedExpr]) -> String {
    format!("[{}]", format_exprs(exprs))
}

fn tell_function_call(func: &str, args: &[AnalyzedExpr]) -> String {
    format!("{}({})", func, format_exprs(args))
}

fn tell_method_call(receiver: &AnalyzedExpr, method: &str, args: &[AnalyzedExpr]) -> String {
    format!("{}.{}({})", tell_expr(receiver), method, format_exprs(args))
}

fn tell_struct_instantiation(
    type_name: &str,
    fields: &[smol_str::SmolStr],
    args: &[AnalyzedExpr],
) -> String {
    let mut buf = String::with_capacity(fields.len() * 16);
    for (i, (f, a)) in fields.iter().zip(args.iter()).enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        let _ = write!(&mut buf, "{}: {}", f, tell_expr(a));
    }
    format!("{} {{ {} }}", type_name, buf)
}

fn tell_lambda(
    params: &[smol_str::SmolStr],
    body: &AnalyzedExpr,
    capture_mode: &CaptureMode,
) -> String {
    let mode = match capture_mode {
        CaptureMode::Borrow => "",
        CaptureMode::Move => "move ",
    };
    format!("{}|{}| {}", mode, params.join(", "), tell_expr(body))
}""": """fn tell_verb_call_into(verb: &str, args: &[AnalyzedExpr], buf: &mut String) {
    let _ = write!(buf, "{}(", verb);
    format_exprs(args, buf);
    buf.push(')');
}

fn tell_array_literal_into(exprs: &[AnalyzedExpr], buf: &mut String) {
    buf.push('[');
    format_exprs(exprs, buf);
    buf.push(']');
}

fn tell_function_call_into(func: &str, args: &[AnalyzedExpr], buf: &mut String) {
    let _ = write!(buf, "{}(", func);
    format_exprs(args, buf);
    buf.push(')');
}

fn tell_method_call_into(receiver: &AnalyzedExpr, method: &str, args: &[AnalyzedExpr], buf: &mut String) {
    tell_expr_into(receiver, buf);
    let _ = write!(buf, ".{}(", method);
    format_exprs(args, buf);
    buf.push(')');
}

fn tell_struct_instantiation_into(
    type_name: &str,
    fields: &[smol_str::SmolStr],
    args: &[AnalyzedExpr],
    buf: &mut String
) {
    let _ = write!(buf, "{} {{ ", type_name);
    for (i, (f, a)) in fields.iter().zip(args.iter()).enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        let _ = write!(buf, "{}: ", f);
        tell_expr_into(a, buf);
    }
    buf.push_str(" }");
}

fn tell_lambda_into(
    params: &[smol_str::SmolStr],
    body: &AnalyzedExpr,
    capture_mode: &CaptureMode,
    buf: &mut String
) {
    let mode = match capture_mode {
        CaptureMode::Borrow => "",
        CaptureMode::Move => "move ",
    };
    let _ = write!(buf, "{}|{}| ", mode, params.join(", "));
    tell_expr_into(body, buf);
}""",

    # tell_type
    """fn tell_type(ty: &GlossaType) -> String {
    match ty {
        GlossaType::Number => "Number".to_string(),
        GlossaType::String => "String".to_string(),
        GlossaType::Boolean => "Bool".to_string(),
        GlossaType::List(inner) => format!("[{}]", tell_type(inner)),
        GlossaType::Set(inner) => format!("Set<{}>", tell_type(inner)),
        GlossaType::Map(k, v) => format!("Map<{}, {}>", tell_type(k), tell_type(v)),
        GlossaType::Option(inner) => format!("Option<{}>", tell_type(inner)),
        GlossaType::Result(ok, err) => format!("Result<{}, {}>", tell_type(ok), tell_type(err)),
        GlossaType::Struct { name, .. } => name.to_string(),
        GlossaType::Function { params, returns } => {
            format!("Fn({}) -> {}", format_types(params), tell_type(returns))
        }
        GlossaType::Unit => "()".to_string(),
        GlossaType::Unknown => "?".to_string(),
    }
}""": """fn tell_type(ty: &GlossaType) -> String {
    let mut buf = String::with_capacity(32);
    tell_type_into(ty, &mut buf);
    buf
}

fn tell_type_into(ty: &GlossaType, buf: &mut String) {
    match ty {
        GlossaType::Number => buf.push_str("Number"),
        GlossaType::String => buf.push_str("String"),
        GlossaType::Boolean => buf.push_str("Bool"),
        GlossaType::List(inner) => {
            buf.push('[');
            tell_type_into(inner, buf);
            buf.push(']');
        }
        GlossaType::Set(inner) => {
            buf.push_str("Set<");
            tell_type_into(inner, buf);
            buf.push('>');
        }
        GlossaType::Map(k, v) => {
            buf.push_str("Map<");
            tell_type_into(k, buf);
            buf.push_str(", ");
            tell_type_into(v, buf);
            buf.push('>');
        }
        GlossaType::Option(inner) => {
            buf.push_str("Option<");
            tell_type_into(inner, buf);
            buf.push('>');
        }
        GlossaType::Result(ok, err) => {
            buf.push_str("Result<");
            tell_type_into(ok, buf);
            buf.push_str(", ");
            tell_type_into(err, buf);
            buf.push('>');
        }
        GlossaType::Struct { name, .. } => buf.push_str(name),
        GlossaType::Function { params, returns } => {
            buf.push_str("Fn(");
            format_types(params, buf);
            buf.push_str(") -> ");
            tell_type_into(returns, buf);
        }
        GlossaType::Unit => buf.push_str("()"),
        GlossaType::Unknown => buf.push('?'),
    }
}""",

    # format_exprs -> to_string update on caller places
    """let script = format!("Proclaim: {}", format_exprs(exprs));""": """let mut expr_str = String::with_capacity(exprs.len() * 16); format_exprs(exprs, &mut expr_str); let script = format!("Proclaim: {}", expr_str);""",

    """let script = format!("Do: {}", format_exprs(exprs));""": """let mut expr_str = String::with_capacity(exprs.len() * 16); format_exprs(exprs, &mut expr_str); let script = format!("Do: {}", expr_str);""",

    """let script = format!("Query oracle: {}", format_exprs(exprs));""": """let mut expr_str = String::with_capacity(exprs.len() * 16); format_exprs(exprs, &mut expr_str); let script = format!("Query oracle: {}", expr_str);""",

    """let script = format!("If {} is true, then:", tell_expr(condition));""": """let mut expr_str = String::with_capacity(32); tell_expr_into(condition, &mut expr_str); let script = format!("If {} is true, then:", expr_str);""",

    """let script = format!("While {} holds true:", tell_expr(condition));""": """let mut expr_str = String::with_capacity(32); tell_expr_into(condition, &mut expr_str); let script = format!("While {} holds true:", expr_str);""",

    """let script = format!("For each `{}` in {}:", variable, tell_expr(iterator));""": """let mut expr_str = String::with_capacity(32); tell_expr_into(iterator, &mut expr_str); let script = format!("For each `{}` in {}:", variable, expr_str);""",

    """let script = format!("Match on {}:", tell_expr(scrutinee));""": """let mut expr_str = String::with_capacity(32); tell_expr_into(scrutinee, &mut expr_str); let script = format!("Match on {}:", expr_str);""",

    """let case_script = format!("Case {}:", tell_expr(pat));""": """let mut pat_str = String::with_capacity(32); tell_expr_into(pat, &mut pat_str); let case_script = format!("Case {}:", pat_str);""",

    """let script = if let Some(v) = value {
        format!("Return {}.", tell_expr(v))
    } else {""": """let script = if let Some(v) = value {
        let mut v_str = String::with_capacity(32);
        tell_expr_into(v, &mut v_str);
        format!("Return {}.", v_str)
    } else {""",

    """let script = format!("Let `{}` be {}.", name, tell_expr(value));""": """let mut val_str = String::with_capacity(32); tell_expr_into(value, &mut val_str); let script = format!("Let `{}` be {}.", name, val_str);""",

    """let script = format!("Update `{}` to {}.", name, tell_expr(value));""": """let mut val_str = String::with_capacity(32); tell_expr_into(value, &mut val_str); let script = format!("Update `{}` to {}.", name, val_str);""",

}

with open("src/tools/narrator.rs", "w") as f:
    f.write(replace_all(content, replacements))
