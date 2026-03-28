//! Name resolution and scope tracking
//!
//! This module acts as the "Memory" of the ΓΛΩΣΣΑ compiler during semantic analysis.
//! It tracks the identity, type, and lifecycle of every symbol (variables, functions,
//! custom types, and traits) across lexical scopes.
//!
//! # The Architecture
//!
//! The resolver uses a stack-based lexical environment:
//! * **[`Scope`]**: The entire environment, containing a stack of [`ScopeLevel`]s.
//! * **[`ScopeLevel`]**: A single dictionary (using `FxHashMap` for speed) mapping names to [`Binding`]s.
//! * **[`ScopeGuard`]**: An RAII (Resource Acquisition Is Initialization) guard returned by [`Scope::enter_scope`].
//!   When the guard goes out of scope, it automatically drops the deepest `ScopeLevel`.
//!
//! This design guarantees that variable shadowing works correctly and that symbols
//! are strictly confined to the block where they are defined, preventing leakage.
//!
//! # Example
//!
//! ```rust
//! use glossa::semantic::Scope;
//! use glossa::semantic::GlossaType;
//!
//! let mut scope = Scope::new();
//! scope.define("ἡλικία", GlossaType::Number); // Let age be a Number
//!
//! {
//!     let mut inner_scope = scope.enter_scope();
//!     inner_scope.define("ὄνομα", GlossaType::String); // Name exists only here
//! } // `inner_scope` is dropped, removing "ὄνομα"
//!
//! assert!(scope.lookup("ἡλικία").is_some());
//! assert!(scope.lookup("ὄνομα").is_none());
//! ```

use crate::semantic::GlossaType;
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

/// A scope level containing symbol bindings
#[derive(Debug, Clone, Default)]
struct ScopeLevel {
    /// Variables defined in this scope.
    /// ⚡ Bolt Optimization: Uses `FxHashMap` instead of the standard `HashMap`
    /// to avoid cryptographic hashing overhead for fast, deterministic lookups of interned strings.
    variables: FxHashMap<SmolStr, Binding>,
    /// Functions defined in this scope.
    functions: FxHashMap<SmolStr, FunctionSignature>,
    /// Types defined in this scope.
    types: FxHashMap<SmolStr, GlossaType>,
    /// Traits defined in this scope.
    traits: FxHashMap<SmolStr, crate::semantic::model::TraitDef>,
    /// Trait implementations in this scope
    trait_impls: Vec<crate::semantic::model::TraitImpl>,
}

impl ScopeLevel {
    fn new() -> Self {
        Self::default()
    }
}

/// A scope containing variable bindings.
///
/// This struct represents the entire lexical environment of a program at a given point
/// in time. It holds a stack of internal levels, allowing for variable shadowing
/// and nested block scopes.
///
/// # Examples
///
/// ```rust
/// use glossa::semantic::Scope;
/// use glossa::semantic::GlossaType;
///
/// let mut scope = Scope::new();
/// scope.define("ξ", GlossaType::Number);
///
/// assert!(scope.is_defined("ξ"));
/// assert_eq!(scope.lookup("ξ"), Some(&GlossaType::Number));
/// ```
#[derive(Debug, Clone)]
pub struct Scope {
    levels: Vec<ScopeLevel>,
}

/// A function signature for tracking defined functions.
///
/// Functions are tracked across the semantic scope so that they can be
/// resolved and their types can be validated when called.
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// The normalized Greek name of the function (typically the verb lemma).
    pub name: SmolStr,
    /// The ordered list of expected parameter [`GlossaType`]s.
    pub param_types: Vec<GlossaType>,
    /// The expected return [`GlossaType`], or `None` if the function yields no value (void).
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

/// A tracked variable binding with type and metadata.
///
/// Bindings are resolved when a variable is used (e.g., retrieving its value).
/// They contain the underlying Rust-compatible [`GlossaType`] for the object
/// and other mutable flags that power compiler warnings and optimizations.
#[derive(Debug, Clone)]
pub struct Binding {
    /// The normalized variable name, acting as the key in the lookup.
    pub name: SmolStr,
    /// The underlying data type of the variable.
    pub glossa_type: GlossaType,
    /// Whether the binding's value can be changed after definition.
    pub mutable: bool,
    /// Tracks if the variable was ever accessed (used for emitting unused variable warnings).
    pub used: bool,
}

impl Scope {
    /// Initializes a new lexical environment.
    ///
    /// The scope starts with a single, root `ScopeLevel`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::semantic::Scope;
    ///
    /// let scope = Scope::new();
    /// ```
    pub fn new() -> Self {
        Scope {
            levels: vec![ScopeLevel::new()],
        }
    }

    /// Enter a new scope level
    fn enter(&mut self) {
        self.levels.push(ScopeLevel::new());
    }

    /// Creates a nested lexical scope and returns a RAII [`ScopeGuard`].
    ///
    /// The returned guard allows defining symbols that only exist within the block.
    /// When the guard goes out of scope and is dropped, the inner scope level
    /// is automatically destroyed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::semantic::Scope;
    /// use glossa::semantic::GlossaType;
    ///
    /// let mut scope = Scope::new();
    /// scope.define("a", GlossaType::Number); // Parent level
    ///
    /// {
    ///     // Enters child scope level
    ///     let mut child_scope = scope.enter_scope();
    ///     child_scope.define("b", GlossaType::String);
    ///
    ///     assert!(child_scope.is_defined("a")); // Inherits parent scope
    ///     assert!(child_scope.is_defined("b")); // Defines own scope
    /// } // `child_scope` is dropped, child level is destroyed
    ///
    /// assert!(scope.is_defined("a"));
    /// assert!(!scope.is_defined("b")); // "b" no longer exists
    /// ```
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
        self.lookup_function(name).is_some()
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
            if let Some(def) = level.traits.get(name) {
                return Some(def);
            }
        }
        None
    }

    /// Register a trait implementation
    pub fn register_trait_impl(&mut self, impl_def: crate::semantic::model::TraitImpl) {
        self.current_level().trait_impls.push(impl_def);
    }


    /// Proves whether a specific form has learned to embody a particular behavior of a character.
    ///
    /// It seeks a `TraitImpl` that binds the type to the trait's name, and checks if
    /// the trait defines that behavior.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use glossa::semantic::Scope;
    /// use glossa::semantic::{TraitImpl, TraitDef, AnalyzedMethod};
    /// use glossa::semantic::GlossaType;
    ///
    /// let mut scope = Scope::new();
    ///
    /// let method = AnalyzedMethod {
    ///     name: "speak".into(),
    ///     params: vec![],
    ///     body: None,
    ///     return_type: None,
    /// };
    ///
    /// let trait_def = TraitDef {
    ///     name: "Speaker".into(),
    ///     methods: vec![method],
    /// };
    /// scope.define_trait("Speaker", trait_def);
    ///
    /// let trait_impl = TraitImpl {
    ///     trait_name: "Speaker".into(),
    ///     type_name: "Human".into(),
    /// };
    /// scope.register_trait_impl(trait_impl);
    ///
    /// assert!(scope.has_trait_method("Human", "speak").is_some());
    /// assert!(scope.has_trait_method("Dog", "speak").is_none());
    /// ```
    pub fn has_trait_method(&self, type_name: &str, method_name: &str) -> Option<SmolStr> {
        for level in self.levels.iter().rev() {
            for trait_impl in &level.trait_impls {
                if trait_impl.type_name != type_name {
                    continue;
                }
                // Check if the trait has this method
                if let Some(trait_def) = self.lookup_trait(&trait_impl.trait_name) {
                    let has_method = trait_def.methods.iter().any(|m| m.name == method_name);
                    if has_method {
                        return Some(trait_impl.trait_name.clone());
                    }
                }
            }
        }
        None
    }

    /// Define a new binding in this scope
    pub fn define(&mut self, name: impl Into<SmolStr>, glossa_type: GlossaType) {
        let name = name.into();
        self.current_level().variables.insert(
            name.clone(),
            Binding {
                name,
                glossa_type,
                mutable: false,
                used: false,
            },
        );
    }

    /// Defines a mutable variable in the current, deepest lexical scope level.
    ///
    /// If a symbol with the same name already exists in an outer scope, the new definition
    /// will shadow the older one until the current `ScopeLevel` is dropped.
    pub fn define_mut(&mut self, name: impl Into<SmolStr>, glossa_type: GlossaType) {
        let name = name.into();
        self.current_level().variables.insert(
            name.clone(),
            Binding {
                name,
                glossa_type,
                mutable: true,
                used: false,
            },
        );
    }

    /// Helper to look up any variable
    fn lookup_variable(&self, name: &str) -> Option<&Binding> {
        for level in self.levels.iter().rev() {
            if let Some(binding) = level.variables.get(name) {
                return Some(binding);
            }
        }
        None
    }

    /// Discovers the true nature ([`GlossaType`]) of an entity by its name.
    ///
    /// The search pierces through the veils of scope, beginning at the current
    /// reality and traveling outward until it finds the origin of the entity.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use glossa::semantic::Scope;
    /// use glossa::semantic::GlossaType;
    ///
    /// let mut scope = Scope::new();
    /// scope.define("x", GlossaType::Number);
    ///
    /// assert!(matches!(scope.lookup("x"), Some(GlossaType::Number)));
    /// assert!(scope.lookup("y").is_none());
    /// ```
    pub fn lookup(&self, name: &str) -> Option<&GlossaType> {
        self.lookup_variable(name).map(|b| &b.glossa_type)
    }

    /// Unveils the full story of an entity, revealing not just its type but its mortality (mutability)
    /// and whether its potential has been realized (used).
    ///
    /// Like `lookup`, it journeys through the nested spheres of scope to find the true source.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use glossa::semantic::Scope;
    /// use glossa::semantic::GlossaType;
    ///
    /// let mut scope = Scope::new();
    /// scope.define("y", GlossaType::String);
    ///
    /// let binding = scope.lookup_binding("y").unwrap();
    /// assert_eq!(binding.name, "y");
    /// ```
    pub fn lookup_binding(&self, name: &str) -> Option<&Binding> {
        self.lookup_variable(name)
    }

    /// Determines if an entity was born within the current, immediate reality.
    ///
    /// This ignores the whispers of ancestors in outer scopes, focusing only on
    /// the present boundary.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use glossa::semantic::Scope;
    /// use glossa::semantic::GlossaType;
    ///
    /// let mut scope = Scope::new();
    /// scope.define("a", GlossaType::Boolean);
    ///
    /// {
    ///     let mut inner = scope.enter_scope();
    ///     assert!(!inner.is_defined_locally("a")); // "a" is an ancestor, not local
    ///     inner.define("b", GlossaType::Number);
    ///     assert!(inner.is_defined_locally("b"));
    /// }
    /// ```
    pub fn is_defined_locally(&self, name: &str) -> bool {
        self.levels
            .last()
            .map(|l| l.variables.contains_key(name))
            .unwrap_or(false)
    }

    /// Senses whether a name holds any meaning across the entirety of known existence.
    ///
    /// It searches variables, actions (functions), forms (types), and characters (traits).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use glossa::semantic::Scope;
    /// use glossa::semantic::GlossaType;
    ///
    /// let mut scope = Scope::new();
    /// scope.define("fate", GlossaType::Number);
    ///
    /// assert!(scope.is_defined("fate"));
    /// assert!(!scope.is_defined("chance"));
    /// ```
    pub fn is_defined(&self, name: &str) -> bool {
        self.lookup_variable(name).is_some()
            || self.lookup_function(name).is_some()
            || self.lookup_type(name).is_some()
            || self.lookup_trait(name).is_some()
    }

    /// Summons every known entity and its story ([`Binding`]) into the light.
    ///
    /// This gathers the truths from all accessible spheres of scope.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use glossa::semantic::Scope;
    /// use glossa::semantic::GlossaType;
    ///
    /// let mut scope = Scope::new();
    /// scope.define("alpha", GlossaType::Number);
    ///
    /// let all_bindings: Vec<_> = scope.bindings().collect();
    /// assert_eq!(all_bindings.len(), 1);
    /// assert_eq!(all_bindings[0].0, "alpha");
    /// ```
    pub fn bindings(&self) -> impl Iterator<Item = (&SmolStr, &Binding)> {
        self.levels.iter().flat_map(|l| l.variables.iter())
    }

    /// Mark a binding as used
    pub fn mark_used(&mut self, name: &str) {
        for level in self.levels.iter_mut().rev() {
            if let Some(binding) = level.variables.get_mut(name) {
                binding.used = true;
                return;
            }
        }
    }

    /// Gathers those entities whose potential was brought into existence but never realized.
    ///
    /// Used by the compiler to whisper warnings of forgotten truths.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use glossa::semantic::Scope;
    /// use glossa::semantic::GlossaType;
    ///
    /// let mut scope = Scope::new();
    /// scope.define("forgotten", GlossaType::Number);
    ///
    /// let unused = scope.unused_bindings();
    /// assert_eq!(unused.len(), 1);
    /// assert_eq!(unused[0].name, "forgotten");
    /// ```
    pub fn unused_bindings(&self) -> Vec<&Binding> {
        self.levels
            .iter()
            .flat_map(|l| l.variables.values())
            .filter(|b| !b.used)
            .collect()
    }

    /// Recites the names and signatures of all known actions ([`FunctionSignature`]) that can shape reality.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use glossa::semantic::Scope;
    /// use glossa::semantic::GlossaType;
    ///
    /// let mut scope = Scope::new();
    /// scope.define_function("create", vec![], Some(GlossaType::Number));
    ///
    /// let all_funcs: Vec<_> = scope.functions().collect();
    /// assert_eq!(all_funcs.len(), 1);
    /// assert_eq!(all_funcs[0].name, "create");
    /// ```
    pub fn functions(&self) -> impl Iterator<Item = &FunctionSignature> {
        self.levels.iter().flat_map(|l| l.functions.values())
    }

    /// Reveals the catalog of all forms ([`GlossaType`]) that reality can take within this scope.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use glossa::semantic::Scope;
    /// use glossa::semantic::GlossaType;
    ///
    /// let mut scope = Scope::new();
    /// scope.define_type("Human", GlossaType::String); // simplified
    ///
    /// let all_types: Vec<_> = scope.types().collect();
    /// assert_eq!(all_types.len(), 1);
    /// assert_eq!(all_types[0].0, "Human");
    /// ```
    pub fn types(&self) -> impl Iterator<Item = (&SmolStr, &GlossaType)> {
        self.levels.iter().flat_map(|l| l.types.iter())
    }

    /// Unveils the ideals and characters ([`crate::semantic::model::TraitDef`]) that forms may strive to embody.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use glossa::semantic::Scope;
    /// use glossa::semantic::TraitDef;
    ///
    /// let mut scope = Scope::new();
    /// let trait_def = TraitDef { name: "Mortal".into(), methods: vec![] };
    /// scope.define_trait("Mortal", trait_def);
    ///
    /// let all_traits: Vec<_> = scope.traits().collect();
    /// assert_eq!(all_traits.len(), 1);
    /// assert_eq!(all_traits[0].0, "Mortal");
    /// ```
    pub fn traits(&self) -> impl Iterator<Item = (&SmolStr, &crate::semantic::model::TraitDef)> {
        self.levels.iter().flat_map(|l| l.traits.iter())
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
    fn test_scope_exit_underflow_protection() {
        let mut scope = Scope::new();
        // A new scope starts with 1 level
        assert_eq!(scope.levels.len(), 1);

        // Attempting to exit when already at the root level should be a no-op
        scope.exit();
        scope.exit();
        scope.exit();

        // Must still have 1 level (preventing .expect() panic in current_level())
        assert_eq!(scope.levels.len(), 1);

        // Verify it still works normally
        scope.define("ξ".to_string(), GlossaType::Number);
        assert!(scope.is_defined("ξ"));
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
        // This is a new test ensuring separate namespace behavior
        let mut scope = Scope::new();
        scope.define_type("Τ".to_string(), GlossaType::Number);

        // Should find as type
        assert!(scope.lookup_type("Τ").is_some());

        // Define variable with same name
        scope.define("Τ".to_string(), GlossaType::String);

        // Should find as variable
        assert!(scope.lookup("Τ").is_some());

        // Should STILL find as type (separate namespace)
        assert!(scope.lookup_type("Τ").is_some());
    }

    #[test]
    fn test_scope_trait_coverage() {
        use crate::semantic::model::TraitDef;
        let mut scope = Scope::new();
        let trait_name = "Δεικτόν";
        let trait_def = TraitDef {
            name: trait_name.into(),
            methods: vec![],
        };

        scope.define_trait(trait_name.to_string(), trait_def);

        assert!(scope.lookup_trait(trait_name).is_some());
        assert!(scope.traits().any(|(k, _)| k == trait_name));

        // Ensure lookup_symbol covers Trait branch
        assert!(scope.is_defined(trait_name));
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

        // Define variable with same name
        scope.define(name.to_string(), GlossaType::Number);

        // Should still be a function (separate namespace)
        assert!(scope.is_function(name));
        assert!(scope.lookup(name).is_some());
    }
}
