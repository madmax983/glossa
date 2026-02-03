//! Name resolution and scope tracking
//!
//! Manages variable bindings and scope for ΓΛΩΣΣΑ programs.

use crate::semantic::GlossaType;
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

/// A scope level containing variable bindings
#[derive(Debug, Clone, Default)]
struct ScopeLevel {
    /// Variable bindings in this scope
    bindings: FxHashMap<SmolStr, Binding>,
    /// Function definitions in this scope
    functions: FxHashMap<SmolStr, FunctionSignature>,
    /// Type definitions in this scope
    types: FxHashMap<SmolStr, GlossaType>,
    /// Trait definitions in this scope
    traits: FxHashMap<SmolStr, crate::semantic::model::TraitDef>,
    /// Trait implementations in this scope
    trait_impls: Vec<crate::semantic::model::TraitImpl>,
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
    pub fn enter(&mut self) {
        self.levels.push(ScopeLevel::new());
    }

    /// Enter a new scope level and return a RAII guard that exits it on drop
    pub fn enter_scope(&mut self) -> ScopeGuard<'_> {
        self.enter();
        ScopeGuard { scope: self }
    }

    /// Exit the current scope level
    pub fn exit(&mut self) {
        if self.levels.len() > 1 {
            self.levels.pop();
        }
    }

    /// Create a child scope with this scope as parent
    pub fn child(&self) -> Self {
        let mut new_scope = self.clone();
        new_scope.enter();
        new_scope
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
        self.current_level().functions.insert(
            name.clone(),
            FunctionSignature {
                name,
                param_types,
                return_type,
            },
        );
    }

    /// Check if a name is a defined function
    pub fn is_function(&self, name: &str) -> bool {
        for level in self.levels.iter().rev() {
            if level.functions.contains_key(name) {
                return true;
            }
        }
        false
    }

    /// Look up a function signature
    pub fn lookup_function(&self, name: &str) -> Option<&FunctionSignature> {
        for level in self.levels.iter().rev() {
            if let Some(sig) = level.functions.get(name) {
                return Some(sig);
            }
        }
        None
    }

    /// Define a type in this scope
    pub fn define_type(&mut self, name: impl Into<SmolStr>, glossa_type: GlossaType) {
        self.current_level().types.insert(name.into(), glossa_type);
    }

    /// Look up a type by name
    pub fn lookup_type(&self, name: &str) -> Option<&GlossaType> {
        for level in self.levels.iter().rev() {
            if let Some(ty) = level.types.get(name) {
                return Some(ty);
            }
        }
        None
    }

    /// Define a trait in this scope
    pub fn define_trait(
        &mut self,
        name: impl Into<SmolStr>,
        trait_def: crate::semantic::model::TraitDef,
    ) {
        self.current_level().traits.insert(name.into(), trait_def);
    }

    /// Look up a trait by name
    pub fn lookup_trait(&self, name: &str) -> Option<&crate::semantic::model::TraitDef> {
        for level in self.levels.iter().rev() {
            if let Some(trait_def) = level.traits.get(name) {
                return Some(trait_def);
            }
        }
        None
    }

    /// Register a trait implementation
    pub fn register_trait_impl(&mut self, impl_def: crate::semantic::model::TraitImpl) {
        self.current_level().trait_impls.push(impl_def);
    }

    /// Look up a trait implementation for a given type and trait
    pub fn lookup_trait_impl(
        &self,
        type_name: &str,
        trait_name: &str,
    ) -> Option<&crate::semantic::model::TraitImpl> {
        for level in self.levels.iter().rev() {
            for impl_def in &level.trait_impls {
                if impl_def.type_name == type_name && impl_def.trait_name == trait_name {
                    return Some(impl_def);
                }
            }
        }
        None
    }

    /// Check if a type has a trait method with the given name
    pub fn has_trait_method(&self, type_name: &str, method_name: &str) -> bool {
        for level in self.levels.iter().rev() {
            for trait_impl in &level.trait_impls {
                if trait_impl.type_name != type_name {
                    continue;
                }
                // Check if the trait has this method
                if let Some(trait_def) = self.lookup_trait(&trait_impl.trait_name) {
                    let has_method = trait_def.methods.iter().any(|m| m.name == method_name);
                    if has_method {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Define a new binding in this scope
    pub fn define(&mut self, name: impl Into<SmolStr>, glossa_type: GlossaType) {
        let name = name.into();
        self.current_level().bindings.insert(
            name.clone(),
            Binding {
                name,
                glossa_type,
                mutable: false,
                used: false,
            },
        );
    }

    /// Define a mutable binding
    pub fn define_mut(&mut self, name: impl Into<SmolStr>, glossa_type: GlossaType) {
        let name = name.into();
        self.current_level().bindings.insert(
            name.clone(),
            Binding {
                name,
                glossa_type,
                mutable: true,
                used: false,
            },
        );
    }

    /// Look up a binding by name, searching parent scopes
    pub fn lookup(&self, name: &str) -> Option<&GlossaType> {
        for level in self.levels.iter().rev() {
            if let Some(binding) = level.bindings.get(name) {
                return Some(&binding.glossa_type);
            }
        }
        None
    }

    /// Look up full binding information by name, searching parent scopes
    pub fn lookup_binding(&self, name: &str) -> Option<&Binding> {
        for level in self.levels.iter().rev() {
            if let Some(binding) = level.bindings.get(name) {
                return Some(binding);
            }
        }
        None
    }

    /// Check if a name is defined in this scope (not parents)
    pub fn is_defined_locally(&self, name: &str) -> bool {
        self.levels
            .last()
            .map(|l| l.bindings.contains_key(name))
            .unwrap_or(false)
    }

    /// Check if a name is defined anywhere in scope chain
    pub fn is_defined(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    /// Get all bindings in this scope (from all levels)
    pub fn bindings(&self) -> impl Iterator<Item = (&SmolStr, &Binding)> {
        self.levels.iter().flat_map(|l| l.bindings.iter())
    }

    /// Mark a binding as used
    pub fn mark_used(&mut self, name: &str) {
        for level in self.levels.iter_mut().rev() {
            if let Some(binding) = level.bindings.get_mut(name) {
                binding.used = true;
                return;
            }
        }
    }

    /// Get unused bindings (for warnings)
    pub fn unused_bindings(&self) -> Vec<&Binding> {
        self.levels
            .iter()
            .flat_map(|l| l.bindings.values())
            .filter(|b| !b.used)
            .collect()
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

        let child = parent.child();
        assert!(child.is_defined("ξ"));
        assert_eq!(child.lookup("ξ"), Some(&GlossaType::Number));
    }

    #[test]
    fn test_child_scope_shadows() {
        let mut parent = Scope::new();
        parent.define("ξ".to_string(), GlossaType::Number);

        let mut child = parent.child();
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

        let child = parent.child();
        let binding = child.lookup_binding("π").unwrap();
        assert_eq!(binding.name, "π");
        assert!(binding.mutable);
    }

    #[test]
    fn test_lookup_binding_not_found() {
        let scope = Scope::new();
        assert!(scope.lookup_binding("ζ").is_none());
    }
}
