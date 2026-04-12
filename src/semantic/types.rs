//! Type system for ΓΛΩΣΣΑ
//!
//! This module defines the [`GlossaType`] enum, which represents the type system of the language.
//!
//! # Philosophy
//!
//! ΓΛΩΣΣΑ maps programming concepts to Ancient Greek philosophical and grammatical constructs:
//!
//! * **Nouns as Types**: Basic types are derived from Greek nouns (e.g., `ἀριθμός` for Number, `ὄνομα` for Name/String).
//! * **Moods as Wrappers**:
//!     * The **Optative Mood** (expressing wish/possibility) maps to `Option<T>` (a value that "might be").
//!     * The **Noun `Ἀποτέλεσμα`** (outcome/result) maps to `Result<T, E>`.
//! * **Free Word Order**: Type compatibility is determined by structure, not just name (though nominal typing is used for structs).

use crate::morphology::Gender;
use smol_str::SmolStr;

/// Types in ΓΛΩΣΣΑ
///
/// This enum represents the type system of the language.
/// It maps directly to Rust types but uses Greek terminology.
///
/// # Type Compatibility
///
/// Types are checked for compatibility using [`GlossaType::is_compatible`].
/// `GlossaType::Unknown` acts as a wildcard that matches any type (useful during inference).
///
/// # Examples
///
/// ```
/// use glossa::semantic::GlossaType;
///
/// // Create simple types
/// let num_type = GlossaType::Number;
/// let str_type = GlossaType::String;
///
/// // Create complex nested types
/// let list_of_numbers = GlossaType::List(Box::new(GlossaType::Number));
///
/// // Check type compatibility
/// assert!(num_type.is_compatible(&GlossaType::Number));
/// assert!(!num_type.is_compatible(&str_type));
///
/// // The Unknown type acts as a wildcard, compatible with anything
/// assert!(GlossaType::Unknown.is_compatible(&str_type));
/// assert!(num_type.is_compatible(&GlossaType::Unknown));
/// ```
#[derive(PartialEq, Eq, Hash)]
pub enum GlossaType {
    /// **ἀριθμός** (Number) - 64-bit signed integer (`i64`)
    ///
    /// # Examples
    /// `1`, `42`, `-5`, `μηδέν` (zero)
    Number,

    /// **ὄνομα** (Name/String) - UTF-8 string (`String`)
    ///
    /// # Examples
    /// `«χαῖρε»` ("hello")
    String,

    /// **ἀληθές/ψεῦδος** (Boolean) - `bool`
    ///
    /// # Examples
    /// `ἀληθές` (true), `ψεῦδος` (false)
    Boolean,

    /// **λίστη** (List) - Dynamic array (`Vec<T>`)
    ///
    /// # Examples
    /// `[1, 2, 3]` (`List<Number>`)
    List(Box<GlossaType>),

    /// **σύνολον** (Set) - Hash set (`HashSet<T>`)
    ///
    /// Stores unique elements.
    Set(Box<GlossaType>),

    /// **χάρτης** (Map) - Hash map (`HashMap<K, V>`)
    ///
    /// Key-value storage.
    Map(Box<GlossaType>, Box<GlossaType>),

    /// `Option<T>` - value that might not exist
    ///
    /// Expressed in ΓΛΩΣΣΑ via the **optative mood** (εὑρεθείη "might be found").
    /// The optative mood in Ancient Greek expresses wish, possibility, or potential,
    /// making it a natural fit for optional values.
    ///
    /// # Examples
    /// - `τί` (ti) → `Some(value)` ("something")
    /// - `οὐδέν` (ouden) → `None` ("nothing")
    /// - `;` after optative verb → propagates `None` (like Rust's `?`)
    Option(Box<GlossaType>),

    /// `Result<T, E>` - value or error
    ///
    /// Expressed in ΓΛΩΣΣΑ via **ἀποτέλεσμα** (apotelasma) "result/outcome".
    /// Uses disjunctive patterns to distinguish success from failure.
    ///
    /// # Examples
    /// - `ἐπιτυχία` (epitychia) → `Ok(value)` ("success")
    /// - `σφάλμα` (sphalma) → `Err(error)` ("error/mistake")
    /// - `;` after Result expression → propagates `Err` (like Rust's `?`)
    Result(Box<GlossaType>, Box<GlossaType>),

    /// **εἶδος** (Form/Type) - User-defined struct
    ///
    /// Named types defined by the user (e.g., `εἶδος Χρήστης`).
    Struct {
        /// The name (`ὄνομα`) given to this specific form of data.
        name: SmolStr,
        /// The grammatical gender dictating how this type interacts with adjectives and articles.
        gender: Gender,
        /// The internal composition that defines what it means to be this type.
        fields: Vec<(SmolStr, GlossaType)>,
    },

    /// **ἔργον** (Function) - Function signature
    ///
    /// Represents the type of a function, including parameter and return types.
    Function {
        /// The required offerings (inputs) needed to invoke this action.
        params: Vec<GlossaType>,
        /// The ultimate outcome (`ἀποτέλεσμα`) produced by the action.
        returns: Box<GlossaType>,
    },

    /// **οὐδέν** (Nothing) - Unit type `()`
    ///
    /// Represents the absence of a value (void).
    Unit,

    /// **ἄγνωστον** (Unknown) - Type inference placeholder
    ///
    /// Used when the type cannot yet be determined. Acts as a wildcard in compatibility checks.
    Unknown,
}

impl Clone for GlossaType {
    fn clone(&self) -> Self {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            GlossaType::Unit => GlossaType::Unit,
            GlossaType::Number => GlossaType::Number,
            GlossaType::String => GlossaType::String,
            GlossaType::Boolean => GlossaType::Boolean,
            GlossaType::Function { params, returns } => GlossaType::Function {
                params: params.clone(),
                returns: returns.clone(),
            },
            GlossaType::Struct {
                name,
                gender,
                fields,
            } => GlossaType::Struct {
                name: name.clone(),
                gender: *gender,
                fields: fields.clone(),
            },
            GlossaType::List(t) => GlossaType::List(t.clone()),
            GlossaType::Map(key, value) => GlossaType::Map(key.clone(), value.clone()),
            GlossaType::Set(t) => GlossaType::Set(t.clone()),
            GlossaType::Option(t) => GlossaType::Option(t.clone()),
            GlossaType::Result(ok, err) => GlossaType::Result(ok.clone(), err.clone()),
            GlossaType::Unknown => GlossaType::Unknown,
        })
    }
}

impl Drop for GlossaType {
    fn drop(&mut self) {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            GlossaType::Function { params, returns } => {
                let _ = std::mem::take(params);
                let _ = std::mem::replace(returns, Box::new(GlossaType::Unit));
            }
            GlossaType::Struct { fields, .. } => {
                let _ = std::mem::take(fields);
            }
            GlossaType::List(t) | GlossaType::Set(t) | GlossaType::Option(t) => {
                let _ = std::mem::replace(&mut **t, GlossaType::Unit);
            }
            GlossaType::Map(key, value) => {
                let _ = std::mem::replace(&mut **key, GlossaType::Unit);
                let _ = std::mem::replace(&mut **value, GlossaType::Unit);
            }
            GlossaType::Result(ok, err) => {
                let _ = std::mem::replace(&mut **ok, GlossaType::Unit);
                let _ = std::mem::replace(&mut **err, GlossaType::Unit);
            }
            _ => {}
        })
    }
}

impl std::fmt::Debug for GlossaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            GlossaType::Number => f.debug_tuple("Number").finish(),
            GlossaType::String => f.debug_tuple("String").finish(),
            GlossaType::Boolean => f.debug_tuple("Boolean").finish(),
            GlossaType::List(inner) => f.debug_tuple("List").field(inner).finish(),
            GlossaType::Set(inner) => f.debug_tuple("Set").field(inner).finish(),
            GlossaType::Map(key, value) => f.debug_tuple("Map").field(key).field(value).finish(),
            GlossaType::Option(inner) => f.debug_tuple("Option").field(inner).finish(),
            GlossaType::Result(ok, err) => f.debug_tuple("Result").field(ok).field(err).finish(),
            GlossaType::Struct {
                name,
                gender,
                fields,
            } => f
                .debug_struct("Struct")
                .field("name", name)
                .field("gender", gender)
                .field("fields", fields)
                .finish(),
            GlossaType::Function { params, returns } => f
                .debug_struct("Function")
                .field("params", params)
                .field("returns", returns)
                .finish(),
            GlossaType::Unit => f.debug_tuple("Unit").finish(),
            GlossaType::Unknown => f.debug_tuple("Unknown").finish(),
        })
    }
}

impl std::fmt::Display for GlossaType {
    /// Formats the type using its Greek name.
    ///
    /// # Examples
    ///
    /// ```
    /// use glossa::semantic::GlossaType;
    ///
    /// assert_eq!(format!("{}", GlossaType::Number), "Ἀριθμός");
    /// assert_eq!(format!("{}", GlossaType::String), "Ὄνομα");
    /// assert_eq!(
    ///     format!("{}", GlossaType::List(Box::new(GlossaType::Number))),
    ///     "Λίστη<Ἀριθμός>"
    /// );
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlossaType::Number => write!(f, "Ἀριθμός"),
            GlossaType::String => write!(f, "Ὄνομα"),
            GlossaType::Boolean => write!(f, "Ἀληθές/Ψεῦδος"),
            GlossaType::List(inner) => write!(f, "Λίστη<{}>", inner),
            GlossaType::Set(inner) => write!(f, "Σύνολον<{}>", inner),
            GlossaType::Map(k, v) => write!(f, "Χάρτης<{}, {}>", k, v),
            GlossaType::Option(inner) => write!(f, "Εὑρεθείη<{}>", inner),
            GlossaType::Result(ok, err) => write!(f, "Ἀποτέλεσμα<{}, {}>", ok, err),
            GlossaType::Struct { name, .. } => write!(f, "Εἶδος {}", name),
            GlossaType::Function { params, returns } => {
                write!(f, "Ἔργον(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", returns)
            }
            GlossaType::Unit => write!(f, "Οὐδέν"),
            GlossaType::Unknown => write!(f, "Ἄγνωστον"),
        }
    }
}

impl GlossaType {
    /// Get the Greek name for this type
    pub fn to_greek(&self) -> &'static str {
        match self {
            GlossaType::Number => "ἀριθμός",
            GlossaType::String => "ὄνομα",
            GlossaType::Boolean => "ἀληθές",
            GlossaType::List(_) => "λίστη",
            GlossaType::Set(_) => "σύνολον",
            GlossaType::Map(_, _) => "χάρτης",
            GlossaType::Option(_) => "εὑρεθείη", // "might be found" (optative)
            GlossaType::Result(_, _) => "ἀποτέλεσμα", // "result/outcome"
            GlossaType::Unit => "οὐδέν",
            GlossaType::Struct { .. } => "εἶδος",
            GlossaType::Function { .. } => "ἔργον",
            GlossaType::Unknown => "ἄγνωστον",
        }
    }

    /// Check if this type is compatible with another
    pub fn is_compatible(&self, other: &GlossaType) -> bool {
        match (self, other) {
            (GlossaType::Unknown, _) | (_, GlossaType::Unknown) => true,
            (GlossaType::List(a), GlossaType::List(b)) => a.is_compatible(b),
            (GlossaType::Set(a), GlossaType::Set(b)) => a.is_compatible(b),
            (GlossaType::Map(k1, v1), GlossaType::Map(k2, v2)) => {
                k1.is_compatible(k2) && v1.is_compatible(v2)
            }
            (GlossaType::Option(a), GlossaType::Option(b)) => a.is_compatible(b),
            (GlossaType::Result(ok1, err1), GlossaType::Result(ok2, err2)) => {
                ok1.is_compatible(ok2) && err1.is_compatible(err2)
            }
            _ => self == other,
        }
    }
}

/// Detect built-in collection types (HashSet, HashMap)
///
/// Returns a tuple of (Rust collection name, GlossaType).
///
/// # Examples
///
/// ```
/// use glossa::semantic::detect_collection_type;
/// use glossa::semantic::GlossaType;
///
/// let (name, ty) = detect_collection_type("χαρτης").unwrap();
/// assert_eq!(name, "HashMap");
/// assert!(matches!(ty, GlossaType::Map(..)));
/// ```
pub fn detect_collection_type(type_name: &str) -> Option<(&'static str, GlossaType)> {
    match type_name {
        "συνολον" => Some(("HashSet", GlossaType::Set(Box::new(GlossaType::Unknown)))),
        "χαρτης" => Some((
            "HashMap",
            GlossaType::Map(Box::new(GlossaType::Unknown), Box::new(GlossaType::Unknown)),
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_to_greek_full() {
        assert_eq!(GlossaType::Number.to_greek(), "ἀριθμός");
        assert_eq!(GlossaType::String.to_greek(), "ὄνομα");
        assert_eq!(GlossaType::Boolean.to_greek(), "ἀληθές");
        assert_eq!(
            GlossaType::List(Box::new(GlossaType::Number)).to_greek(),
            "λίστη"
        );
        assert_eq!(
            GlossaType::Set(Box::new(GlossaType::Number)).to_greek(),
            "σύνολον"
        );
        assert_eq!(
            GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number)).to_greek(),
            "χάρτης"
        );
        assert_eq!(
            GlossaType::Option(Box::new(GlossaType::Number)).to_greek(),
            "εὑρεθείη"
        );
        assert_eq!(
            GlossaType::Result(Box::new(GlossaType::Number), Box::new(GlossaType::String))
                .to_greek(),
            "ἀποτέλεσμα"
        );
        assert_eq!(GlossaType::Unit.to_greek(), "οὐδέν");
        assert_eq!(
            GlossaType::Struct {
                name: "test".into(),
                gender: crate::morphology::Gender::Neuter,
                fields: vec![]
            }
            .to_greek(),
            "εἶδος"
        );
        assert_eq!(
            GlossaType::Function {
                params: vec![],
                returns: Box::new(GlossaType::Unit)
            }
            .to_greek(),
            "ἔργον"
        );
        assert_eq!(GlossaType::Unknown.to_greek(), "ἄγνωστον");
    }

    #[test]
    fn test_format_function_type_multiple_params() {
        let func = GlossaType::Function {
            params: vec![GlossaType::Number, GlossaType::String],
            returns: Box::new(GlossaType::Boolean),
        };
        assert_eq!(
            format!("{}", func),
            "Ἔργον(Ἀριθμός, Ὄνομα) -> Ἀληθές/Ψεῦδος"
        );
    }

    #[test]
    fn test_type_compatibility() {
        assert!(GlossaType::Number.is_compatible(&GlossaType::Number));
        assert!(!GlossaType::Number.is_compatible(&GlossaType::String));
        assert!(GlossaType::Unknown.is_compatible(&GlossaType::Number));
        assert!(GlossaType::Number.is_compatible(&GlossaType::Unknown));

        assert!(
            GlossaType::List(Box::new(GlossaType::Number))
                .is_compatible(&GlossaType::List(Box::new(GlossaType::Number)))
        );
        assert!(
            !GlossaType::List(Box::new(GlossaType::Number))
                .is_compatible(&GlossaType::List(Box::new(GlossaType::String)))
        );

        assert!(
            GlossaType::Set(Box::new(GlossaType::Number))
                .is_compatible(&GlossaType::Set(Box::new(GlossaType::Number)))
        );
        assert!(
            !GlossaType::Set(Box::new(GlossaType::Number))
                .is_compatible(&GlossaType::Set(Box::new(GlossaType::String)))
        );

        assert!(
            GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number))
                .is_compatible(&GlossaType::Map(
                    Box::new(GlossaType::String),
                    Box::new(GlossaType::Number)
                ))
        );
        assert!(
            !GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number))
                .is_compatible(&GlossaType::Map(
                    Box::new(GlossaType::Number),
                    Box::new(GlossaType::String)
                ))
        );

        assert!(
            GlossaType::Option(Box::new(GlossaType::Number))
                .is_compatible(&GlossaType::Option(Box::new(GlossaType::Number)))
        );
        assert!(
            !GlossaType::Option(Box::new(GlossaType::Number))
                .is_compatible(&GlossaType::Option(Box::new(GlossaType::String)))
        );

        assert!(
            GlossaType::Result(Box::new(GlossaType::Number), Box::new(GlossaType::String))
                .is_compatible(&GlossaType::Result(
                    Box::new(GlossaType::Number),
                    Box::new(GlossaType::String)
                ))
        );
        assert!(
            !GlossaType::Result(Box::new(GlossaType::Number), Box::new(GlossaType::String))
                .is_compatible(&GlossaType::Result(
                    Box::new(GlossaType::String),
                    Box::new(GlossaType::Number)
                ))
        );
    }

    #[test]
    fn test_detect_collection_type_full() {
        assert!(detect_collection_type("συνολον").is_some());
        assert!(detect_collection_type("χαρτης").is_some());
        assert!(detect_collection_type("other").is_none());

        let result = detect_collection_type("συνολον");
        assert_eq!(result.unwrap().0, "HashSet");

        let result = detect_collection_type("χαρτης");
        assert_eq!(result.unwrap().0, "HashMap");

        let result = detect_collection_type("Invalid");
        assert!(result.is_none());
    }

    #[test]
    fn test_display_formatting() {
        assert_eq!(format!("{}", GlossaType::Number), "Ἀριθμός");
        assert_eq!(format!("{}", GlossaType::String), "Ὄνομα");
        assert_eq!(format!("{}", GlossaType::Boolean), "Ἀληθές/Ψεῦδος");
        assert_eq!(
            format!("{}", GlossaType::List(Box::new(GlossaType::Number))),
            "Λίστη<Ἀριθμός>"
        );
        assert_eq!(
            format!("{}", GlossaType::Set(Box::new(GlossaType::Number))),
            "Σύνολον<Ἀριθμός>"
        );
        assert_eq!(
            format!(
                "{}",
                GlossaType::Map(
                    Box::new(GlossaType::String),
                    Box::new(GlossaType::Option(Box::new(GlossaType::Number)))
                )
            ),
            "Χάρτης<Ὄνομα, Εὑρεθείη<Ἀριθμός>>"
        );
        assert_eq!(
            format!(
                "{}",
                GlossaType::Result(Box::new(GlossaType::Unit), Box::new(GlossaType::String))
            ),
            "Ἀποτέλεσμα<Οὐδέν, Ὄνομα>"
        );
        assert_eq!(
            format!(
                "{}",
                GlossaType::Struct {
                    name: "User".into(),
                    gender: Gender::Masculine,
                    fields: vec![]
                }
            ),
            "Εἶδος User"
        );
        assert_eq!(
            format!(
                "{}",
                GlossaType::Function {
                    params: vec![GlossaType::Number, GlossaType::String],
                    returns: Box::new(GlossaType::Boolean)
                }
            ),
            "Ἔργον(Ἀριθμός, Ὄνομα) -> Ἀληθές/Ψεῦδος"
        );
        assert_eq!(format!("{}", GlossaType::Unit), "Οὐδέν");
        assert_eq!(format!("{}", GlossaType::Unknown), "Ἄγνωστον");
    }
}
