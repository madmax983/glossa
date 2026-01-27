//! AST node definitions for ΓΛΩΣΣΑ
//!
//! These nodes capture the structure of a GLOSSA program,
//! preserving both original Greek text and normalized forms.


/// A complete GLOSSA program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// A single statement, ending with . (statement) or ? (query)
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// A regular statement with clauses
    Regular {
        clauses: Vec<Clause>,
        is_query: bool,
    },
    /// A type definition statement
    TypeDefinition(TypeDef),
    /// A trait definition statement
    TraitDefinition(TraitDef),
    /// A trait implementation statement
    TraitImpl(TraitImplDef),
}

/// A type definition (struct)
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: Word,
    pub fields: Vec<FieldDecl>,
}

/// A field declaration in a type
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub name: Word,
    pub type_name: Word,
}

/// A trait definition
#[derive(Debug, Clone, PartialEq)]
pub struct TraitDef {
    pub name: Word,
    pub methods: Vec<TraitMethodDecl>,
}

/// A method declaration in a trait
#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethodDecl {
    pub name: Word,
    pub params: Vec<FieldDecl>,
    pub is_default: bool,
    pub body: Option<Vec<Statement>>,
}

/// A trait implementation
#[derive(Debug, Clone, PartialEq)]
pub struct TraitImplDef {
    pub type_name: Word,
    pub trait_name: Word,
    pub methods: Vec<ImplMethodDef>,
}

/// A method implementation in a trait impl
#[derive(Debug, Clone, PartialEq)]
pub struct ImplMethodDef {
    pub name: Word,
    pub params: Vec<FieldDecl>,
    pub body: Vec<Statement>,
}

/// A clause within a statement (comma-separated)
#[derive(Debug, Clone, PartialEq)]
pub struct Clause {
    /// Expressions in this clause (chained with middle dot)
    pub expressions: Vec<Expr>,
}

impl Statement {
    /// Get all expressions flattened (for backwards compatibility)
    pub fn expressions(&self) -> Box<dyn Iterator<Item = &Expr> + '_> {
        match self {
            Statement::Regular { clauses, .. } => {
                Box::new(clauses.iter().flat_map(|c| c.expressions.iter()))
            }
            Statement::TypeDefinition(_) => Box::new(std::iter::empty()),
            Statement::TraitDefinition(_) => Box::new(std::iter::empty()),
            Statement::TraitImpl(_) => Box::new(std::iter::empty()),
        }
    }

    /// Check if this is a query statement
    pub fn is_query(&self) -> bool {
        match self {
            Statement::Regular { is_query, .. } => *is_query,
            Statement::TypeDefinition(_) => false,
            Statement::TraitDefinition(_) => false,
            Statement::TraitImpl(_) => false,
        }
    }

    /// Get clauses if this is a regular statement
    pub fn clauses(&self) -> &[Clause] {
        match self {
            Statement::Regular { clauses, .. } => clauses,
            Statement::TypeDefinition(_) => &[],
            Statement::TraitDefinition(_) => &[],
            Statement::TraitImpl(_) => &[],
        }
    }
}

/// An expression in GLOSSA
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A string literal: «text»
    StringLiteral(String),

    /// A number literal
    NumberLiteral(i64),

    /// A boolean literal: ἀληθές or ψεῦδος
    BooleanLiteral(bool),

    /// An array literal: [1, 2, 3]
    ArrayLiteral(Vec<Expr>),

    /// Index access: array[index]
    IndexAccess {
        array: Box<Expr>,
        index: Box<Expr>,
    },

    /// A single Greek word with morphological information
    Word(Word),

    /// Multiple terms forming a phrase
    Phrase(Vec<Expr>),

    /// A property access (genitive construction)
    /// e.g., χρήστου ὄνομα = "the name of the user"
    PropertyAccess {
        owner: Box<Expr>,
        property: Box<Expr>,
    },

    /// A function/verb call
    Call {
        verb: Word,
        arguments: Vec<Expr>,
    },

    /// Variable binding (ἔστω construction)
    Binding {
        name: Word,
        value: Box<Expr>,
    },

    /// Binary operation (arithmetic, comparison, boolean)
    /// e.g., x μεῖζον y → x > y
    BinOp {
        left: Box<Expr>,
        op: BinOperator,
        right: Box<Expr>,
    },

    /// Unary operation (negation)
    /// e.g., οὐκ x → !x
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expr>,
    },

    /// A block of statements in braces { ... }
    Block(Vec<Statement>),

    /// A lambda/closure constructed from a participle
    /// e.g., γράφων → |x| x.write()
    Lambda {
        kind: LambdaKind,
        verb_lemma: String,
        implicit_param: bool,  // true if parameter inferred from context
    },
}

/// Lambda kind derived from participle tense/voice
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LambdaKind {
    /// Present participle - streaming/borrowing closure
    /// e.g., γράφων → |x| body
    Streaming,

    /// Aorist participle - one-shot/consuming closure
    /// e.g., γράψας → move |x| body
    OneShot,

    /// Perfect participle - memoized/cached closure
    /// e.g., γεγραμμένος → cached |x| body
    Memoized,
}

/// Capture mode for closures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    /// Borrow captured variables (default for present participles)
    Borrow,

    /// Move captured variables (for aorist participles)
    Move,

    /// Memoize result (for perfect participles)
    Memoize,
}

/// Binary operators in GLOSSA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOperator {
    // Arithmetic
    Add,  // ἄθροισμα
    Sub,  // διαφορά
    Mul,  // γινόμενον
    Div,  // μέρος
    Mod,  // ὑπόλοιπον

    // Comparison
    Eq,   // ἴσον
    Ne,   // ἄνισον
    Lt,   // ἔλαττον
    Le,   // ἔλαττον ἢ ἴσον
    Gt,   // μεῖζον
    Ge,   // μεῖζον ἢ ἴσον

    // Boolean
    And,  // καί
    Or,   // ἤ
}

/// Unary operators in GLOSSA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,  // οὐ/οὐκ/οὐχ
    Neg,  // arithmetic negation
}

/// A Greek word with original and normalized forms
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    /// Original text with diacritics
    pub original: String,
    /// Normalized (lowercase, no diacritics)
    pub normalized: String,
}

impl Word {
    pub fn new(original: impl Into<String>) -> Self {
        let original = original.into();
        let normalized = crate::grammar::normalize_greek(&original);
        Word { original, normalized }
    }
}

/// Analyzed word with morphological information
#[derive(Debug, Clone, PartialEq)]
pub struct AnalyzedWord {
    pub word: Word,
    pub case: Option<crate::morphology::Case>,
    pub number: Option<crate::morphology::Number>,
    pub gender: Option<crate::morphology::Gender>,
    pub person: Option<crate::morphology::Person>,
    pub tense: Option<crate::morphology::Tense>,
    pub mood: Option<crate::morphology::Mood>,
    pub voice: Option<crate::morphology::Voice>,
}

impl From<Word> for AnalyzedWord {
    fn from(word: Word) -> Self {
        AnalyzedWord {
            word,
            case: None,
            number: None,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
        }
    }
}
