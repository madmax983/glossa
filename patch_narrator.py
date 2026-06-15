with open("src/tools/narrator.rs", "r") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if line.startswith("pub(crate) fn tell_expr"):
        start_idx = i
    if line.startswith("#[cfg(test)]"):
        end_idx = i
        break

replacement = """pub(crate) fn tell_expr(expr: &AnalyzedExpr) -> String {
    let mut result = String::with_capacity(64);
    write_tell_expr(expr, &mut result).unwrap();
    result
}

fn write_tell_expr(expr: &AnalyzedExpr, out: &mut String) -> std::fmt::Result {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => write!(out, "\\"{}\\"", s),
        AnalyzedExprKind::NumberLiteral(n) => write!(out, "{}", n),
        AnalyzedExprKind::BooleanLiteral(b) => write!(out, "{}", b),
        AnalyzedExprKind::Variable(name) => write!(out, "`{}`", name),
        AnalyzedExprKind::VerbCall { verb, args } => {
            write!(out, "{}(", verb)?;
            write_format_exprs(args, out)?;
            write!(out, ")")
        }
        AnalyzedExprKind::BinOp { left, op, right } => {
            write!(out, "(")?;
            write_tell_expr(left, out)?;
            write!(out, " {:?} ", op)?;
            write_tell_expr(right, out)?;
            write!(out, ")")
        }
        AnalyzedExprKind::UnaryOp { op, operand } => {
            write!(out, "({:?} ", op)?;
            write_tell_expr(operand, out)?;
            write!(out, ")")
        }
        AnalyzedExprKind::Range {
            start,
            end,
            inclusive,
        } => {
            let range_op = if *inclusive { "..=" } else { ".." };
            write_tell_expr(start, out)?;
            write!(out, "{}", range_op)?;
            write_tell_expr(end, out)
        }
        AnalyzedExprKind::Some(e) => {
            write!(out, "Some(")?;
            write_tell_expr(e, out)?;
            write!(out, ")")
        }
        AnalyzedExprKind::None => write!(out, "None"),
        AnalyzedExprKind::Ok(e) => {
            write!(out, "Ok(")?;
            write_tell_expr(e, out)?;
            write!(out, ")")
        }
        AnalyzedExprKind::Err(e) => {
            write!(out, "Err(")?;
            write_tell_expr(e, out)?;
            write!(out, ")")
        }
        AnalyzedExprKind::Unwrap(e) => {
            write_tell_expr(e, out)?;
            write!(out, "!")
        }
        AnalyzedExprKind::Try(e) => {
            write_tell_expr(e, out)?;
            write!(out, "?")
        }
        AnalyzedExprKind::IndexAccess { array, index } => {
            write_tell_expr(array, out)?;
            write!(out, "[")?;
            write_tell_expr(index, out)?;
            write!(out, "]")
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => {
            write!(out, "[")?;
            write_format_exprs(exprs, out)?;
            write!(out, "]")
        }
        AnalyzedExprKind::FunctionCall { func, args } => {
            write!(out, "{}(", func)?;
            write_format_exprs(args, out)?;
            write!(out, ")")
        }
        AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } => {
            write_tell_expr(receiver, out)?;
            write!(out, ".{}(", method)?;
            write_format_exprs(args, out)?;
            write!(out, ")")
        }
        AnalyzedExprKind::StructInstantiation { type_name, fields, args } => {
            let mut buf = String::with_capacity(fields.len() * 32);
            for (i, f) in fields.iter().enumerate() {
                if i > 0 {
                    buf.push_str(", ");
                }
                if let Some(arg) = args.get(i) {
                    let mut arg_str = String::with_capacity(32);
                    write_tell_expr(arg, &mut arg_str).unwrap();
                    buf.push_str(&format!("{}: {}", f, arg_str));
                } else {
                    buf.push_str(&format!("{}: _", f));
                }
            }
            write!(out, "{} {{ {} }}", type_name, buf)
        }
        AnalyzedExprKind::Lambda {
            params,
            body,
            capture_mode,
        } => {
            let mode = match capture_mode {
                CaptureMode::Borrow => "",
                CaptureMode::Move => "move ",
            };
            write!(out, "{}|{}| ", mode, params.join(", "))?;
            write_tell_expr(body, out)
        }
        AnalyzedExprKind::CollectionNew { collection_type } => {
            write!(out, "{}::new()", collection_type)
        }
        AnalyzedExprKind::Assert { condition } => {
            write!(out, "assert(")?;
            write_tell_expr(condition, out)?;
            write!(out, ")")
        }
        AnalyzedExprKind::AssertEq { left, right } => {
            write!(out, "assert_eq(")?;
            write_tell_expr(left, out)?;
            write!(out, ", ")?;
            write_tell_expr(right, out)?;
            write!(out, ")")
        }
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            write_tell_expr(owner, out)?;
            write!(out, ".{}", property)
        }
    }
}

fn write_format_exprs(exprs: &[AnalyzedExpr], out: &mut String) -> std::fmt::Result {
    for (i, expr) in exprs.iter().enumerate() {
        if i > 0 {
            write!(out, ", ")?;
        }
        write_tell_expr(expr, out)?;
    }
    Ok(())
}

fn tell_type(ty: &GlossaType) -> String {
    let mut result = String::with_capacity(32);
    write_tell_type(ty, &mut result).unwrap();
    result
}

fn write_tell_type(ty: &GlossaType, out: &mut String) -> std::fmt::Result {
    match ty {
        GlossaType::Number => write!(out, "Number"),
        GlossaType::String => write!(out, "String"),
        GlossaType::Boolean => write!(out, "Bool"),
        GlossaType::List(inner) => {
            write!(out, "[")?;
            write_tell_type(inner, out)?;
            write!(out, "]")
        }
        GlossaType::Set(inner) => {
            write!(out, "Set<")?;
            write_tell_type(inner, out)?;
            write!(out, ">")
        }
        GlossaType::Map(k, v) => {
            write!(out, "Map<")?;
            write_tell_type(k, out)?;
            write!(out, ", ")?;
            write_tell_type(v, out)?;
            write!(out, ">")
        }
        GlossaType::Option(inner) => {
            write!(out, "Option<")?;
            write_tell_type(inner, out)?;
            write!(out, ">")
        }
        GlossaType::Result(ok, err) => {
            write!(out, "Result<")?;
            write_tell_type(ok, out)?;
            write!(out, ", ")?;
            write_tell_type(err, out)?;
            write!(out, ">")
        }
        GlossaType::Struct { name, .. } => write!(out, "{}", name),
        GlossaType::Function { params, returns } => {
            write!(out, "Fn(")?;
            for (i, p) in params.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ")?;
                }
                write_tell_type(p, out)?;
            }
            write!(out, ") -> ")?;
            write_tell_type(returns, out)
        }
        GlossaType::Unit => write!(out, "()"),
        GlossaType::Unknown => write!(out, "?"),
    }
}

"""

new_lines = lines[:start_idx] + [replacement] + lines[end_idx:]

with open("src/tools/narrator.rs", "w") as f:
    f.writelines(new_lines)

with open("src/tools/narrator.rs", "r") as f:
    content = f.read()

content = content.replace("fn format_types(types: &[GlossaType]) -> String {", "pub fn format_types(types: &[GlossaType]) -> String {")

with open("src/tools/narrator.rs", "w") as f:
    f.write(content)
