import re

with open('src/ast.rs', 'r') as f:
    content = f.read()

content = content.replace("#[derive(Clone, PartialEq)]\npub enum Statement", "pub enum Statement")
content = content.replace("#[derive(Debug, Clone, PartialEq)]\npub struct TypeDef", "#[derive(Debug, PartialEq)]\npub struct TypeDef")
content = content.replace("#[derive(Debug, Clone, PartialEq)]\npub struct FieldDecl", "#[derive(Debug, PartialEq)]\npub struct FieldDecl")
content = content.replace("#[derive(Debug, Clone, PartialEq)]\npub struct TraitDef", "#[derive(Debug, PartialEq)]\npub struct TraitDef")
content = content.replace("#[derive(Debug, Clone, PartialEq)]\npub struct TraitMethodDecl", "#[derive(Debug, PartialEq)]\npub struct TraitMethodDecl")
content = content.replace("#[derive(Debug, Clone, PartialEq)]\npub struct TraitImplDef", "#[derive(Debug, PartialEq)]\npub struct TraitImplDef")
content = content.replace("#[derive(Debug, Clone, PartialEq)]\npub struct ImplMethodDef", "#[derive(Debug, PartialEq)]\npub struct ImplMethodDef")
content = content.replace("#[derive(Debug, Clone, PartialEq)]\npub struct TestDecl", "#[derive(Debug, PartialEq)]\npub struct TestDecl")

clone_drop_impl = """
impl Clone for Statement {
    fn clone(&self) -> Self {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            Statement::Regular { clauses, is_query, is_propagate } => Statement::Regular { clauses: clauses.clone(), is_query: *is_query, is_propagate: *is_propagate },
            Statement::TypeDefinition(t) => Statement::TypeDefinition(t.clone()),
            Statement::TraitDefinition(t) => Statement::TraitDefinition(t.clone()),
            Statement::TraitImpl(t) => Statement::TraitImpl(t.clone()),
            Statement::TestDeclaration(t) => Statement::TestDeclaration(t.clone()),
        })
    }
}

impl PartialEq for Statement {
    fn eq(&self, other: &Self) -> bool {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match (self, other) {
            (Statement::Regular { clauses: c1, is_query: q1, is_propagate: p1 }, Statement::Regular { clauses: c2, is_query: q2, is_propagate: p2 }) => c1 == c2 && q1 == q2 && p1 == p2,
            (Statement::TypeDefinition(t1), Statement::TypeDefinition(t2)) => t1 == t2,
            (Statement::TraitDefinition(t1), Statement::TraitDefinition(t2)) => t1 == t2,
            (Statement::TraitImpl(t1), Statement::TraitImpl(t2)) => t1 == t2,
            (Statement::TestDeclaration(t1), Statement::TestDeclaration(t2)) => t1 == t2,
            _ => false,
        })
    }
}

impl Drop for Statement {
    fn drop(&mut self) {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            match self {
                Statement::Regular { clauses, .. } => { let _ = std::mem::take(clauses); }
                Statement::TypeDefinition(_) => {}
                Statement::TraitDefinition(_) => {}
                Statement::TraitImpl(_) => {}
                Statement::TestDeclaration(_) => {}
            }
        })
    }
}

impl Clone for TypeDef { fn clone(&self) -> Self { stacker::maybe_grow(32 * 1024, 1024 * 1024, || Self { name: self.name.clone(), fields: self.fields.clone() }) } }
impl Clone for FieldDecl { fn clone(&self) -> Self { stacker::maybe_grow(32 * 1024, 1024 * 1024, || Self { name: self.name.clone(), expected_type: self.expected_type.clone() }) } }
impl Clone for TraitDef { fn clone(&self) -> Self { stacker::maybe_grow(32 * 1024, 1024 * 1024, || Self { name: self.name.clone(), methods: self.methods.clone() }) } }
impl Clone for TraitMethodDecl { fn clone(&self) -> Self { stacker::maybe_grow(32 * 1024, 1024 * 1024, || Self { name: self.name.clone(), params: self.params.clone(), body: self.body.clone(), is_default: self.is_default }) } }
impl Clone for TraitImplDef { fn clone(&self) -> Self { stacker::maybe_grow(32 * 1024, 1024 * 1024, || Self { type_name: self.type_name.clone(), trait_name: self.trait_name.clone(), methods: self.methods.clone() }) } }
impl Clone for ImplMethodDef { fn clone(&self) -> Self { stacker::maybe_grow(32 * 1024, 1024 * 1024, || Self { name: self.name.clone(), params: self.params.clone(), body: self.body.clone() }) } }
impl Clone for TestDecl { fn clone(&self) -> Self { stacker::maybe_grow(32 * 1024, 1024 * 1024, || Self { name: self.name.clone(), body: self.body.clone() }) } }
"""

content = content.replace("    TestDeclaration(TestDecl),\n}\n", "    TestDeclaration(TestDecl),\n}\n" + clone_drop_impl)

with open('src/ast.rs', 'w') as f:
    f.write(content)
