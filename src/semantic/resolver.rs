//! Name resolution and scope tracking
//!
//! Manages variable bindings and scope for ΓΛΩΣΣΑ programs.

use crate::semantic::GlossaType;
use smol_str::SmolStr;
use std::collections::HashMap;

/// A unified symbol entry in the scope
#[derive(Debug, Clone)]
pub enum Symbol {
    Variable(Binding),
    Function(FunctionSignature),
    Type(GlossaType),
}

/// A scope level containing symbol bindings
#[derive(Debug, Clone, Default)]
struct ScopeLevel {
    /// Symbols defined in this scope
    symbols: HashMap<SmolStr, Symbol>,
}

impl ScopeLevel {
    fn new() -> Self {
        Self::default()
    }
}

/// A scope containing variable bindings
#[derive(Debug, Clone)]
pub struct Scope {
    levels: Vec<ScopeLevel>,
}

/// A function signature for tracking defined functions
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// The function name (normalized)
    pub name: SmolStr,
    /// Parameter types
    pub param_types: Vec<GlossaType>,
    /// Return type (None for void)
    pub return_type: Option<GlossaType>,
}

/// A RAII guard for a scope level. Exits the scope when dropped.
pub struct ScopeGuard<'a> {
    scope: &'a mut Scope,
}

impl<'a> Drop for ScopeGuard<'a> {
    fn drop(&mut self) {
        self.scope.exit();
    }
}

impl<'a> std::ops::Deref for ScopeGuard<'a> {
    type Target = Scope;
    fn deref(&self) -> &Scope {
        self.scope
    }
}

impl<'a> std::ops::DerefMut for ScopeGuard<'a> {
    fn deref_mut(&mut self) -> &mut Scope {
        self.scope
    }
}

/// A variable binding with type and metadata
#[derive(Debug, Clone)]
pub struct Binding {
    /// The variable name (normalized)
    pub name: SmolStr,
    /// The type of the variable
    pub glossa_type: GlossaType,
    /// Whether this binding is mutable
    pub mutable: bool,
    /// Whether this binding has been used
    pub used: bool,
}

impl Scope {
    /// Create a new empty scope
    pub fn new() -> Self {
        Scope {
            levels: vec![ScopeLevel::new()],
        }
    }

    /// Enter a new scope level
    fn enter(&mut self) {
        self.levels.push(ScopeLevel::new());
    }

    /// Enter a new scope level and return a RAII guard that exits it on drop
    pub fn enter_scope(&mut self) -> ScopeGuard<'_> {
        self.enter();
        ScopeGuard { scope: self }
    }

    /// Exit the current scope level
    fn exit(&mut self) {
        if self.levels.len() > 1 {
            self.levels.pop();
        }
    }

    fn current_level(&mut self) -> &mut ScopeLevel {
        self.levels
            .last_mut()
            .expect("Scope must have at least one level")
    }

    /// Define a function in this scope
    pub fn define_function(
        &mut self,
        name: impl Into<SmolStr>,
        param_types: Vec<GlossaType>,
        return_type: Option<GlossaType>,
    ) {
        let name = name.into();
        self.current_level().symbols.insert(
            name.clone(),
            Symbol::Function(FunctionSignature {
                name,
                param_types,
                return_type,
            }),
        );
    }

    /// Check if a name is a defined function
    pub fn is_function(&self, name: &str) -> bool {
        self.lookup_symbol(name)
            .map(|s| matches!(s, Symbol::Function(_)))
            .unwrap_or(false)
    }

    /// Look up a function signature
    pub fn lookup_function(&self, name: &str) -> Option<&FunctionSignature> {
        match self.lookup_symbol(name) {
            Some(Symbol::Function(sig)) => Some(sig),
            _ => None,
        }
    }

    /// Define a type in this scope
    pub fn define_type(&mut self, name: impl Into<SmolStr>, glossa_type: GlossaType) {
        self.current_level()
            .symbols
            .insert(name.into(), Symbol::Type(glossa_type));
    }

    /// Look up a type by name
    pub fn lookup_type(&self, name: &str) -> Option<&GlossaType> {
        match self.lookup_symbol(name) {
            Some(Symbol::Type(ty)) => Some(ty),
            _ => None,
        }
    }

    /// Define a new binding in this scope
    pub fn define(&mut self, name: impl Into<SmolStr>, glossa_type: GlossaType) {
        let name = name.into();
        self.current_level().symbols.insert(
            name.clone(),
            Symbol::Variable(Binding {
                name,
                glossa_type,
                mutable: false,
                used: false,
            }),
        );
    }

    /// Define a mutable binding
    pub fn define_mut(&mut self, name: impl Into<SmolStr>, glossa_type: GlossaType) {
        let name = name.into();
        self.current_level().symbols.insert(
            name.clone(),
            Symbol::Variable(Binding {
                name,
                glossa_type,
                mutable: true,
                used: false,
            }),
        );
    }

    /// Helper to look up any symbol
    fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        for level in self.levels.iter().rev() {
            if let Some(symbol) = level.symbols.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    /// Look up a binding by name, searching parent scopes
    pub fn lookup(&self, name: &str) -> Option<&GlossaType> {
        match self.lookup_symbol(name) {
            Some(Symbol::Variable(binding)) => Some(&binding.glossa_type),
            _ => None,
        }
    }

    /// Look up full binding information by name, searching parent scopes
    pub fn lookup_binding(&self, name: &str) -> Option<&Binding> {
        match self.lookup_symbol(name) {
            Some(Symbol::Variable(binding)) => Some(binding),
            _ => None,
        }
    }

    /// Check if a name is defined in this scope (not parents)
    pub fn is_defined_locally(&self, name: &str) -> bool {
        self.levels
            .last()
            .map(|l| l.symbols.contains_key(name))
            .unwrap_or(false)
    }

    /// Check if a name is defined anywhere in scope chain
    pub fn is_defined(&self, name: &str) -> bool {
        // We only check for variables with this method typically,
        // but let's check for any symbol as name collision prevents definition
        self.lookup_symbol(name).is_some()
    }

    /// Get all bindings in this scope (from all levels)
    pub fn bindings(&self) -> impl Iterator<Item = (&SmolStr, &Binding)> {
        self.levels.iter().flat_map(|l| {
            l.symbols.iter().filter_map(|(k, v)| match v {
                Symbol::Variable(b) => Some((k, b)),
                _ => None,
            })
        })
    }

    /// Mark a binding as used
    pub fn mark_used(&mut self, name: &str) {
        for level in self.levels.iter_mut().rev() {
            if let Some(Symbol::Variable(binding)) = level.symbols.get_mut(name) {
                binding.used = true;
                return;
            }
        }
    }

    /// Get unused bindings (for warnings)
    pub fn unused_bindings(&self) -> Vec<&Binding> {
        self.levels
            .iter()
            .flat_map(|l| l.symbols.values())
            .filter_map(|s| match s {
                Symbol::Variable(b) => Some(b),
                _ => None,
            })
            .filter(|b| !b.used)
            .collect()
    }

    /// Get all functions defined in this scope
    pub fn functions(&self) -> impl Iterator<Item = &FunctionSignature> {
        self.levels.iter().flat_map(|l| {
            l.symbols.values().filter_map(|s| match s {
                Symbol::Function(f) => Some(f),
                _ => None,
            })
        })
    }

    /// Get all types defined in this scope
    pub fn types(&self) -> impl Iterator<Item = (&SmolStr, &GlossaType)> {
        self.levels.iter().flat_map(|l| {
            l.symbols.iter().filter_map(|(k, v)| match v {
                Symbol::Type(t) => Some((k, t)),
                _ => None,
            })
        })
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_define_and_lookup() {
        let mut scope = Scope::new();
        scope.define("ξ".to_string(), GlossaType::Number);

        assert!(scope.is_defined("ξ"));
        assert_eq!(scope.lookup("ξ"), Some(&GlossaType::Number));
        assert!(!scope.is_defined("υ"));
    }

    #[test]
    fn test_child_scope_inherits() {
        let mut parent = Scope::new();
        parent.define("ξ".to_string(), GlossaType::Number);

        let child = parent.enter_scope();
        assert!(child.is_defined("ξ"));
        assert_eq!(child.lookup("ξ"), Some(&GlossaType::Number));
    }

    #[test]
    fn test_child_scope_shadows() {
        let mut parent = Scope::new();
        parent.define("ξ".to_string(), GlossaType::Number);

        let mut child = parent.enter_scope();
        child.define("ξ".to_string(), GlossaType::String);

        assert_eq!(child.lookup("ξ"), Some(&GlossaType::String));
    }

    #[test]
    fn test_enter_exit_scope() {
        let mut scope = Scope::new();
        scope.define("ξ".to_string(), GlossaType::Number);

        scope.enter();
        assert!(scope.is_defined("ξ")); // Inherits
        scope.define("υ".to_string(), GlossaType::String);
        assert!(scope.is_defined("υ"));

        scope.exit();
        assert!(scope.is_defined("ξ"));
        assert!(!scope.is_defined("υ")); // υ was popped
    }

    #[test]
    fn test_enter_shadowing() {
        let mut scope = Scope::new();
        scope.define("ξ".to_string(), GlossaType::Number);

        scope.enter();
        scope.define("ξ".to_string(), GlossaType::String);
        assert_eq!(scope.lookup("ξ"), Some(&GlossaType::String));

        scope.exit();
        assert_eq!(scope.lookup("ξ"), Some(&GlossaType::Number));
    }

    #[test]
    fn test_mutable_binding() {
        let mut scope = Scope::new();
        scope.define_mut("μ".to_string(), GlossaType::Number);

        let binding = scope.lookup_binding("μ").unwrap();
        assert!(binding.mutable);
    }

    #[test]
    fn test_mark_used() {
        let mut scope = Scope::new();
        scope.define("ξ".to_string(), GlossaType::Number);

        {
            let binding = scope.lookup_binding("ξ").unwrap();
            assert!(!binding.used);
        }

        scope.mark_used("ξ");
        let binding = scope.lookup_binding("ξ").unwrap();
        assert!(binding.used);
    }

    #[test]
    fn test_unused_bindings() {
        let mut scope = Scope::new();
        scope.define("ξ".to_string(), GlossaType::Number);
        scope.define("υ".to_string(), GlossaType::String);
        scope.mark_used("ξ");

        let unused = scope.unused_bindings();
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].name, "υ");
    }

    #[test]
    fn test_lookup_binding_returns_full_binding() {
        let mut scope = Scope::new();
        scope.define_mut("μ".to_string(), GlossaType::Number);

        let binding = scope.lookup_binding("μ").unwrap();
        assert_eq!(binding.name, "μ");
        assert_eq!(binding.glossa_type, GlossaType::Number);
        assert!(binding.mutable);
    }

    #[test]
    fn test_lookup_binding_immutable() {
        let mut scope = Scope::new();
        scope.define("ξ".to_string(), GlossaType::String);

        let binding = scope.lookup_binding("ξ").unwrap();
        assert_eq!(binding.name, "ξ");
        assert_eq!(binding.glossa_type, GlossaType::String);
        assert!(!binding.mutable);
    }

    #[test]
    fn test_lookup_binding_parent_scope() {
        let mut parent = Scope::new();
        parent.define_mut("π".to_string(), GlossaType::Number);

        let child = parent.enter_scope();
        let binding = child.lookup_binding("π").unwrap();
        assert_eq!(binding.name, "π");
        assert!(binding.mutable);
    }

    #[test]
    fn test_lookup_binding_not_found() {
        let scope = Scope::new();
        assert!(scope.lookup_binding("ζ").is_none());
    }

    #[test]
    fn test_shadowing_across_types() {
        // This is a new test ensuring single namespace behavior
        let mut scope = Scope::new();
        scope.define_type("Τ".to_string(), GlossaType::Number);

        // Should find as type
        assert!(scope.lookup_type("Τ").is_some());

        // Shadow with variable
        scope.define("Τ".to_string(), GlossaType::String);

        // Should find as variable
        assert!(scope.lookup("Τ").is_some());

        // Should NOT find as type anymore (shadowed by variable)
        assert!(scope.lookup_type("Τ").is_none());
    }

    #[test]
    fn test_scope_function_coverage() {
        let mut scope = Scope::new();
        let func_name = "λέγε";

        scope.define_function(
            func_name.to_string(),
            vec![GlossaType::String],
            Some(GlossaType::Unit),
        );

        assert!(scope.lookup_function(func_name).is_some());
        assert!(scope.is_function(func_name));
        assert!(scope.functions().any(|f| f.name == func_name));

        // Ensure lookup_symbol covers Function branch
        assert!(scope.is_defined(func_name));
    }

    #[test]
    fn test_scope_type_coverage() {
        let mut scope = Scope::new();
        let type_name = "Χρήστης";

        scope.define_type(type_name.to_string(), GlossaType::Number);

        assert!(scope.lookup_type(type_name).is_some());
        assert!(scope.types().any(|(k, _)| k == type_name));

        // Ensure lookup_symbol covers Type branch
        assert!(scope.is_defined(type_name));
    }

    #[test]
    fn test_scope_mixed_namespace_collisions() {
        let mut scope = Scope::new();
        let name = "κοινόν";

        // Define as function
        scope.define_function(name.to_string(), vec![], None);

        // Verify it is a function
        assert!(scope.is_function(name));

        // Overwrite with variable (shadowing)
        scope.define(name.to_string(), GlossaType::Number);

        // Should now be a variable, not a function (in lookups specific to functions)
        assert!(!scope.is_function(name));
        assert!(scope.lookup(name).is_some());
    }
}
