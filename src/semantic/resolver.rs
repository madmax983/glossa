//! Name resolution and scope tracking
//!
//! Manages variable bindings and scope for ΓΛΩΣΣΑ programs.

use crate::semantic::GlossaType;
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

/// A single level of scope (e.g., a function body, a block)
#[derive(Debug, Clone, Default)]
struct ScopeLevel {
    /// Variable bindings in this scope level
    bindings: FxHashMap<SmolStr, Binding>,
    /// Function definitions in this scope level
    functions: FxHashMap<SmolStr, FunctionSignature>,
    /// Type definitions in this scope level
    types: FxHashMap<SmolStr, GlossaType>,
    /// Trait definitions in this scope level
    traits: FxHashMap<SmolStr, crate::semantic::model::TraitDef>,
    /// Trait implementations in this scope level
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
    /// Stack of scope levels (last is current)
    levels: Vec<ScopeLevel>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
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
    /// Create a new empty scope with a global level
    pub fn new() -> Self {
        Scope {
            levels: vec![ScopeLevel::new()],
        }
    }

    /// Enter a new scope level
    pub fn enter(&mut self) {
        self.levels.push(ScopeLevel::new());
    }

    /// Exit the current scope level
    pub fn exit(&mut self) {
        if self.levels.len() > 1 {
            self.levels.pop();
        } else {
            panic!("Attempted to exit global scope");
        }
    }

    /// Define a function in this scope
    pub fn define_function(
        &mut self,
        name: impl Into<SmolStr>,
        param_types: Vec<GlossaType>,
        return_type: Option<GlossaType>,
    ) {
        let name = name.into();
        self.current_level_mut().functions.insert(
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
        self.current_level_mut()
            .types
            .insert(name.into(), glossa_type);
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
        self.current_level_mut()
            .traits
            .insert(name.into(), trait_def);
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
        self.current_level_mut().trait_impls.push(impl_def);
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
                // Note: We need to lookup trait def, which might be in a different level
                // Optimization: lookup trait def once per trait impl? No, usually traits are top level.
                if let Some(trait_def) = self.lookup_trait(&trait_impl.trait_name) {
                    let has_method = trait_def
                        .required_methods
                        .iter()
                        .any(|m| m.name == method_name)
                        || trait_def
                            .default_methods
                            .iter()
                            .any(|m| m.signature.name == method_name);
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
        self.current_level_mut().bindings.insert(
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
        self.current_level_mut().bindings.insert(
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
        self.current_level().bindings.contains_key(name)
    }

    /// Check if a name is defined anywhere in scope chain
    pub fn is_defined(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    /// Get all bindings in this scope level
    pub fn bindings(&self) -> impl Iterator<Item = (&SmolStr, &Binding)> {
        self.current_level().bindings.iter()
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
        // Only checking current level? Or all levels?
        // Typically warnings are generated when a scope closes.
        // For now, let's return unused bindings from ALL levels to be safe,
        // or just the current level. The original implementation did `bindings.values()`, which was just the current node.
        self.current_level()
            .bindings
            .values()
            .filter(|b| !b.used)
            .collect()
    }

    fn current_level(&self) -> &ScopeLevel {
        self.levels.last().expect("Scope stack is empty")
    }

    fn current_level_mut(&mut self) -> &mut ScopeLevel {
        self.levels.last_mut().expect("Scope stack is empty")
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
        let mut scope = Scope::new();
        scope.define("ξ".to_string(), GlossaType::Number);

        scope.enter();
        assert!(scope.is_defined("ξ"));
        assert_eq!(scope.lookup("ξ"), Some(&GlossaType::Number));
        scope.exit();
    }

    #[test]
    fn test_child_scope_shadows() {
        let mut scope = Scope::new();
        scope.define("ξ".to_string(), GlossaType::Number);

        scope.enter();
        scope.define("ξ".to_string(), GlossaType::String);

        assert_eq!(scope.lookup("ξ"), Some(&GlossaType::String));
        scope.exit();

        // Original should be back
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

        assert!(!scope.lookup_binding("ξ").unwrap().used);
        scope.mark_used("ξ");
        assert!(scope.lookup_binding("ξ").unwrap().used);
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
    #[should_panic(expected = "Attempted to exit global scope")]
    fn test_exit_global_scope_panics() {
        let mut scope = Scope::new();
        scope.exit();
    }
}
