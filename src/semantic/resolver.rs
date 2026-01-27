//! Name resolution and scope tracking
//!
//! Manages variable bindings and scope for ΓΛΩΣΣΑ programs.

use crate::semantic::GlossaType;
use rustc_hash::FxHashMap;

/// A scope containing variable bindings
#[derive(Debug, Clone, Default)]
pub struct Scope {
    /// Variable bindings in this scope
    bindings: FxHashMap<String, Binding>,
    /// Function definitions in this scope
    functions: FxHashMap<String, FunctionSignature>,
    /// Type definitions in this scope
    types: FxHashMap<String, GlossaType>,
    /// Trait definitions in this scope
    traits: FxHashMap<String, crate::semantic::types::TraitDef>,
    /// Trait implementations in this scope
    trait_impls: Vec<crate::semantic::types::TraitImpl>,
    /// Parent scope (for nested scopes)
    parent: Option<Box<Scope>>,
}

/// A function signature for tracking defined functions
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// The function name (normalized)
    pub name: String,
    /// Parameter types
    pub param_types: Vec<GlossaType>,
    /// Return type (None for void)
    pub return_type: Option<GlossaType>,
}

/// A variable binding with type and metadata
#[derive(Debug, Clone)]
pub struct Binding {
    /// The variable name (normalized)
    pub name: String,
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
            bindings: FxHashMap::default(),
            functions: FxHashMap::default(),
            types: FxHashMap::default(),
            traits: FxHashMap::default(),
            trait_impls: Vec::new(),
            parent: None,
        }
    }

    /// Create a child scope with this scope as parent
    pub fn child(&self) -> Self {
        Scope {
            bindings: FxHashMap::default(),
            functions: FxHashMap::default(),
            types: FxHashMap::default(),
            traits: FxHashMap::default(),
            trait_impls: Vec::new(),
            parent: Some(Box::new(self.clone())),
        }
    }

    /// Define a function in this scope
    pub fn define_function(&mut self, name: String, param_types: Vec<GlossaType>, return_type: Option<GlossaType>) {
        self.functions.insert(
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
        if self.functions.contains_key(name) {
            true
        } else if let Some(parent) = &self.parent {
            parent.is_function(name)
        } else {
            false
        }
    }

    /// Look up a function signature
    pub fn lookup_function(&self, name: &str) -> Option<&FunctionSignature> {
        if let Some(sig) = self.functions.get(name) {
            Some(sig)
        } else if let Some(parent) = &self.parent {
            parent.lookup_function(name)
        } else {
            None
        }
    }

    /// Define a type in this scope
    pub fn define_type(&mut self, name: String, glossa_type: GlossaType) {
        self.types.insert(name, glossa_type);
    }

    /// Look up a type by name
    pub fn lookup_type(&self, name: &str) -> Option<&GlossaType> {
        if let Some(ty) = self.types.get(name) {
            Some(ty)
        } else if let Some(parent) = &self.parent {
            parent.lookup_type(name)
        } else {
            None
        }
    }

    /// Define a trait in this scope
    pub fn define_trait(&mut self, name: String, trait_def: crate::semantic::types::TraitDef) {
        self.traits.insert(name, trait_def);
    }

    /// Look up a trait by name
    pub fn lookup_trait(&self, name: &str) -> Option<&crate::semantic::types::TraitDef> {
        if let Some(trait_def) = self.traits.get(name) {
            Some(trait_def)
        } else if let Some(parent) = &self.parent {
            parent.lookup_trait(name)
        } else {
            None
        }
    }

    /// Register a trait implementation
    pub fn register_trait_impl(&mut self, impl_def: crate::semantic::types::TraitImpl) {
        self.trait_impls.push(impl_def);
    }

    /// Look up a trait implementation for a given type and trait
    pub fn lookup_trait_impl(&self, type_name: &str, trait_name: &str) -> Option<&crate::semantic::types::TraitImpl> {
        for impl_def in &self.trait_impls {
            if impl_def.type_name == type_name && impl_def.trait_name == trait_name {
                return Some(impl_def);
            }
        }
        if let Some(parent) = &self.parent {
            parent.lookup_trait_impl(type_name, trait_name)
        } else {
            None
        }
    }

    /// Check if a type has a trait method with the given name
    pub fn has_trait_method(&self, type_name: &str, method_name: &str) -> bool {
        for trait_impl in &self.trait_impls {
            if trait_impl.type_name != type_name {
                continue;
            }
            // Check if the trait has this method
            if let Some(trait_def) = self.lookup_trait(&trait_impl.trait_name) {
                let has_method = trait_def.required_methods.iter().any(|m| m.name == method_name) ||
                                 trait_def.default_methods.iter().any(|m| m.signature.name == method_name);
                if has_method {
                    return true;
                }
            }
        }
        // Check parent scope
        if let Some(parent) = &self.parent {
            parent.has_trait_method(type_name, method_name)
        } else {
            false
        }
    }

    /// Define a new binding in this scope
    pub fn define(&mut self, name: String, glossa_type: GlossaType) {
        self.bindings.insert(
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
    pub fn define_mut(&mut self, name: String, glossa_type: GlossaType) {
        self.bindings.insert(
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
        if let Some(binding) = self.bindings.get(name) {
            Some(&binding.glossa_type)
        } else if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }

    /// Check if a name is defined in this scope (not parents)
    pub fn is_defined_locally(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }

    /// Check if a name is defined anywhere in scope chain
    pub fn is_defined(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    /// Get all bindings in this scope
    pub fn bindings(&self) -> impl Iterator<Item = (&String, &Binding)> {
        self.bindings.iter()
    }

    /// Mark a binding as used
    pub fn mark_used(&mut self, name: &str) {
        if let Some(binding) = self.bindings.get_mut(name) {
            binding.used = true;
        } else if let Some(parent) = &mut self.parent {
            parent.mark_used(name);
        }
    }

    /// Get unused bindings (for warnings)
    pub fn unused_bindings(&self) -> Vec<&Binding> {
        self.bindings
            .values()
            .filter(|b| !b.used)
            .collect()
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
    fn test_mutable_binding() {
        let mut scope = Scope::new();
        scope.define_mut("μ".to_string(), GlossaType::Number);

        let binding = scope.bindings.get("μ").unwrap();
        assert!(binding.mutable);
    }

    #[test]
    fn test_mark_used() {
        let mut scope = Scope::new();
        scope.define("ξ".to_string(), GlossaType::Number);

        assert!(!scope.bindings.get("ξ").unwrap().used);
        scope.mark_used("ξ");
        assert!(scope.bindings.get("ξ").unwrap().used);
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
}
