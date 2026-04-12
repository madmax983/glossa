import re

with open('src/semantic/model.rs', 'r') as f:
    content = f.read()

content = content.replace("#[derive(Clone)]\npub struct AnalyzedExpr {", "pub struct AnalyzedExpr {")
content = content.replace("#[derive(Clone)]\npub enum AnalyzedExprKind {", "pub enum AnalyzedExprKind {")

clone_drop_expr = """
impl Clone for AnalyzedExpr {
    fn clone(&self) -> Self {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || AnalyzedExpr {
            expr: self.expr.clone(),
            glossa_type: self.glossa_type.clone(),
        })
    }
}

impl Drop for AnalyzedExpr {
    fn drop(&mut self) {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            match &mut self.expr {
                AnalyzedExprKind::PropertyAccess { owner, .. } => { let _ = std::mem::replace(&mut **owner, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean }); }
                AnalyzedExprKind::BinOp { left, right, .. } => { let _ = std::mem::replace(&mut **left, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean }); let _ = std::mem::replace(&mut **right, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean }); }
                AnalyzedExprKind::UnaryOp { operand, .. } => { let _ = std::mem::replace(&mut **operand, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean }); }
                AnalyzedExprKind::Range { start, end, .. } => { let _ = std::mem::replace(&mut **start, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean }); let _ = std::mem::replace(&mut **end, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean }); }
                AnalyzedExprKind::Some(v) | AnalyzedExprKind::Ok(v) | AnalyzedExprKind::Err(v) | AnalyzedExprKind::Unwrap(v) | AnalyzedExprKind::Try(v) => { let _ = std::mem::replace(&mut **v, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean }); }
                AnalyzedExprKind::IndexAccess { array, index } => { let _ = std::mem::replace(&mut **array, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean }); let _ = std::mem::replace(&mut **index, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean }); }
                AnalyzedExprKind::MethodCall { receiver, .. } => { let _ = std::mem::replace(&mut **receiver, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean }); }
                AnalyzedExprKind::Lambda { body, .. } => { let _ = std::mem::replace(&mut **body, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean }); }
                AnalyzedExprKind::Assert { condition } => { let _ = std::mem::replace(&mut **condition, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean }); }
                AnalyzedExprKind::AssertEq { left, right } => { let _ = std::mem::replace(&mut **left, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean }); let _ = std::mem::replace(&mut **right, AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(false), glossa_type: crate::semantic::GlossaType::Boolean }); }
                _ => {}
            }
        })
    }
}
"""
content = content.replace("    pub glossa_type: GlossaType,\n}\n", "    pub glossa_type: GlossaType,\n}\n" + clone_drop_expr)

clone_kind = """
impl Clone for AnalyzedExprKind {
    fn clone(&self) -> Self {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            AnalyzedExprKind::StringLiteral(s) => AnalyzedExprKind::StringLiteral(s.clone()),
            AnalyzedExprKind::NumberLiteral(n) => AnalyzedExprKind::NumberLiteral(*n),
            AnalyzedExprKind::BooleanLiteral(b) => AnalyzedExprKind::BooleanLiteral(*b),
            AnalyzedExprKind::Variable(v) => AnalyzedExprKind::Variable(v.clone()),
            AnalyzedExprKind::PropertyAccess { owner, property } => AnalyzedExprKind::PropertyAccess { owner: owner.clone(), property: property.clone() },
            AnalyzedExprKind::VerbCall { verb, args } => AnalyzedExprKind::VerbCall { verb: verb.clone(), args: args.clone() },
            AnalyzedExprKind::BinOp { left, op, right } => AnalyzedExprKind::BinOp { left: left.clone(), op: *op, right: right.clone() },
            AnalyzedExprKind::UnaryOp { op, operand } => AnalyzedExprKind::UnaryOp { op: *op, operand: operand.clone() },
            AnalyzedExprKind::Range { start, end, inclusive } => AnalyzedExprKind::Range { start: start.clone(), end: end.clone(), inclusive: *inclusive },
            AnalyzedExprKind::ArrayLiteral(v) => AnalyzedExprKind::ArrayLiteral(v.clone()),
            AnalyzedExprKind::Some(v) => AnalyzedExprKind::Some(v.clone()),
            AnalyzedExprKind::None => AnalyzedExprKind::None,
            AnalyzedExprKind::Ok(v) => AnalyzedExprKind::Ok(v.clone()),
            AnalyzedExprKind::Err(v) => AnalyzedExprKind::Err(v.clone()),
            AnalyzedExprKind::Unwrap(v) => AnalyzedExprKind::Unwrap(v.clone()),
            AnalyzedExprKind::Try(v) => AnalyzedExprKind::Try(v.clone()),
            AnalyzedExprKind::IndexAccess { array, index } => AnalyzedExprKind::IndexAccess { array: array.clone(), index: index.clone() },
            AnalyzedExprKind::FunctionCall { func, args } => AnalyzedExprKind::FunctionCall { func: func.clone(), args: args.clone() },
            AnalyzedExprKind::MethodCall { receiver, method, args } => AnalyzedExprKind::MethodCall { receiver: receiver.clone(), method: method.clone(), args: args.clone() },
            AnalyzedExprKind::StructInstantiation { type_name, fields, args } => AnalyzedExprKind::StructInstantiation { type_name: type_name.clone(), fields: fields.clone(), args: args.clone() },
            AnalyzedExprKind::Lambda { params, body, capture_mode } => AnalyzedExprKind::Lambda { params: params.clone(), body: body.clone(), capture_mode: *capture_mode },
            AnalyzedExprKind::CollectionNew { collection_type } => AnalyzedExprKind::CollectionNew { collection_type: collection_type.clone() },
            AnalyzedExprKind::Assert { condition } => AnalyzedExprKind::Assert { condition: condition.clone() },
            AnalyzedExprKind::AssertEq { left, right } => AnalyzedExprKind::AssertEq { left: left.clone(), right: right.clone() },
        })
    }
}
"""
content = content.replace("        right: Box<AnalyzedExpr>,\n    },\n}\n", "        right: Box<AnalyzedExpr>,\n    },\n}\n" + clone_kind)


with open('src/semantic/model.rs', 'w') as f:
    f.write(content)
