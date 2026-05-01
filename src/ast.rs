//! Abstract Syntax Tree for ΓΛΩΣΣΑ
//!
//! The AST captures the semantic structure of a GLOSSA program,
//! preserving morphological information from Greek words.
//!
//! # The Tree Structure
//!
//! The AST hierarchy reflects the grammatical structure of the language:
//!
//! * [`Program`]: The root node, containing a list of statements.
//! * [`Statement`]: A sentence, ending with a period (`.`) or query mark (`?` / `;`).
//!   * A statement consists of one or more [`Clause`]s.
//! * [`Clause`]: A comma-separated part of a statement.
//!   * Example: `ὁ ἄνθρωπος, τὸν λόγον λέγει.` (Two clauses: "The man", "says the word").
//! * [`Expr`]: An expression (word, literal, operation).
//!   * [`Expr::Word`]: A raw Greek word with its original and normalized forms.
//!
//! # Design Philosophy
//!
//! Unlike traditional ASTs that might discard surface-level details, the GLOSSA AST
//! preserves the *original* Greek text in [`Word`] nodes. This is crucial for:
//!
//! 1. **Error Reporting**: Using the original polytonic Greek in error messages.
//! 2. **Morphological Analysis**: The semantic phase needs the original form to
//!    distinguish subtle variations if needed.
//!
//! # Example
//!
//! A simple program like `«χαῖρε» λέγε.` produces:
//!
//! ```text
//! Program
//! └── Statement::Regular
//!     └── Clause
//!         ├── Expr::StringLiteral("χαῖρε")
//!         └── Expr::Word("λέγε")
//! ```

use smol_str::SmolStr;

/// A complete ΓΛΩΣΣΑ program
///
/// This is the root node of the Abstract Syntax Tree (AST). It acts as the grand
/// container for every action, definition, and evaluation written by the user.
/// After the lexer and parser construct a raw Concrete Syntax Tree (CST) from strings,
/// the compiler transforms it into this strongly-typed `Program`.
///
/// # Examples
///
/// ```rust,ignore
/// use glossa::parser::parse;
/// use glossa::ast::{Program, Statement};
///
/// // Create a program with a single statement
/// let source = "«χαῖρε κόσμε» λέγε.";
/// let program: Program = parse(source).unwrap();
///
/// assert_eq!(program.statements.len(), 1);
/// assert!(matches!(program.statements[0], Statement::Regular { .. }));
/// ```
#[derive(Clone, PartialEq)]
pub struct Program {
    /// The linear sequence of top-level statements executed from top to bottom.
    pub statements: Vec<Statement>,
}

impl std::fmt::Debug for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            f.debug_struct("Program")
                .field("statements", &self.statements)
                .finish()
        })
    }
}

/// A single statement, ending with . (statement), ? (query), or ; (propagate)
#[derive(Clone, PartialEq)]
pub enum Statement {
    /// A regular statement with clauses
    ///
    /// The most common statement type, representing imperative actions,
    /// bindings, or expressions.
    ///
    /// # Example
    ///
    /// ```glossa
    /// «χαῖρε» λέγε.
    /// ```
    Regular {
        /// The constituent phrases separated by commas, representing distinct actions or subjects in the sentence.
        clauses: Vec<Clause>,
        /// Indicates if the user appended a question mark (`?`), signifying an inquiry about the program state rather than a command.
        is_query: bool,
        /// Indicates if the statement concludes with a semicolon (`;`), dictating that errors or absence of values should bubble up the call stack.
        is_propagate: bool,
    },
    /// A type definition statement
    ///
    /// Defines a new struct type.
    ///
    /// # Example
    ///
    /// ```glossa
    /// εἶδος Χρήστης ὁρίζειν {
    ///     ὄνομα ὀνόματος.
    /// }.
    /// ```
    TypeDefinition(TypeDef),
    /// A trait definition statement
    ///
    /// Defines a new interface (trait).
    ///
    /// # Example
    ///
    /// ```glossa
    /// χαρακτήρ Εκτυπώσιμος ὁρίζειν {
    ///     τύπωσις(εαυτός).
    /// }.
    /// ```
    TraitDefinition(TraitDef),
    /// A trait implementation statement
    ///
    /// Implements a trait for a specific type.
    ///
    /// # Example
    ///
    /// ```glossa
    /// εἶδος Χρήστης τῷ Εκτυπώσιμος ἐμπίπτειν {
    ///     τύπωσις(εαυτός) {
    ///         εαυτοῦ ὄνομα λέγε.
    ///     }
    /// }.
    /// ```
    TraitImpl(TraitImplDef),
    /// A test declaration
    ///
    /// Defines a unit test.
    ///
    /// # Example
    ///
    /// ```glossa
    /// δοκιμή «test name».
    ///     ξ 5 ἰσοῦται.
    /// τέλος.
    /// ```
    TestDeclaration(TestDecl),
}

/// A custom Type Definition (`εἶδος`)
///
/// In ΓΛΩΣΣΑ, users can define their own complex data structures using the `εἶδος` keyword.
/// This struct represents the declaration of such a type (similar to a `struct` in Rust or C),
/// holding its name and the fields that make it up.
///
/// # Examples
///
/// ```rust,ignore,ignore
/// use glossa::parser::parse;
/// use glossa::ast::Statement;
///
/// // Parse a struct definition: "Define type point { x, y }"
/// // The syntax requires fields to be colon-separated and typed in AST,
/// // though simplified in parsing. We show a typical example.
/// let source = "εἶδος σημεῖον ὁρίζειν { ξ, ψ }.";
/// let program = parse(source).unwrap();
///
/// if let Statement::TypeDefinition(type_def) = &program.statements[0] {
///     assert_eq!(type_def.name.normalized.as_str(), "σημειον");
///     assert_eq!(type_def.fields.len(), 2);
/// } else {
///     panic!("Expected TypeDefinition");
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    /// The overarching identifier assigned to this new user-defined archetype.
    pub name: Word,
    /// The structured properties and attributes that instances of this archetype will carry in memory.
    pub fields: Vec<FieldDecl>,
}

/// A property declaration inside a [`TypeDef`]
///
/// This represents a single field defined within an `εἶδος` (Type Definition) block.
/// Unlike dynamically typed languages, ΓΛΩΣΣΑ requires fields to have an explicitly
/// declared type, although it is often omitted in the raw AST if type inference can
/// resolve it later.
///
/// # Examples
///
/// ```rust,ignore
/// use glossa::parser::parse;
/// use glossa::ast::Statement;
///
/// let source = "εἶδος σημεῖον ὁρίζειν { ξ, ψ }.";
/// let program = parse(source).unwrap();
///
/// if let Statement::TypeDefinition(type_def) = &program.statements[0] {
///     let field_x = &type_def.fields[0];
///     assert_eq!(field_x.name.normalized.as_str(), "ξ");
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    /// The identifier by which this specific property is accessed.
    pub name: Word,
    /// The archetype dictating the shape and constraints of the data stored within this property.
    pub type_name: Word,
}

/// A capability contract (`χαρακτήρ`)
///
/// In ΓΛΩΣΣΑ, users define shared behaviors using the `χαρακτήρ` (Trait) keyword.
/// This struct holds the declaration of a trait, ensuring that any type that
/// claims to implement it provides the required actions (`methods`).
///
/// # Examples
///
/// ```rust,ignore
/// use glossa::parser::parse;
/// use glossa::ast::Statement;
///
/// // "Define the 'speaker' trait { speak }"
/// let source = "χαρακτήρ λέκτης ὁρίζειν { λέγειν }.";
/// let program = parse(source).unwrap();
///
/// if let Statement::TraitDefinition(trait_def) = &program.statements[0] {
///     assert_eq!(trait_def.name.normalized.as_str(), "λεκτης");
///     assert_eq!(trait_def.methods.len(), 1);
/// } else {
///     panic!("Expected TraitDefinition");
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TraitDef {
    /// The shared identifier for this collection of required behaviors or capabilities.
    pub name: Word,
    /// The list of required actions that any adhering archetype must know how to perform.
    pub methods: Vec<TraitMethodDecl>,
}

/// A required behavior inside a [`TraitDef`]
///
/// This represents a specific action (method) that forms part of a `χαρακτήρ` (Trait).
/// It lists the expected verb that conforming types must respond to, alongside
/// any expected inputs and potential default implementations.
///
/// # Examples
///
/// ```rust,ignore
/// use glossa::parser::parse;
/// use glossa::ast::Statement;
///
/// let source = "χαρακτήρ λέκτης ὁρίζειν { λέγειν }.";
/// let program = parse(source).unwrap();
///
/// if let Statement::TraitDefinition(trait_def) = &program.statements[0] {
///     let method = &trait_def.methods[0];
///     assert_eq!(method.name.normalized.as_str(), "λεγειν");
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethodDecl {
    /// The identifier for the specific behavior expected by the trait.
    pub name: Word,
    /// The inputs required to perform the action, bound to specific archetypes.
    pub params: Vec<FieldDecl>,
    /// Indicates whether a fallback behavior has been provided for archetypes that do not define their own.
    pub is_default: bool,
    /// The fallback logic provided when `is_default` is enabled, saving the implementor from rewriting standard behavior.
    pub body: Option<Vec<Statement>>,
}

/// A trait implementation block (`ἐφαρμόζειν`)
///
/// This structure links a specific type (`εἶδος`) to a shared behavior (`χαρακτήρ`),
/// providing the concrete logic (`methods`) that satisfies the trait's contract.
///
/// # Examples
///
/// ```rust,ignore
/// use glossa::parser::parse;
/// use glossa::ast::Statement;
///
/// // Parse a trait implementation: "Implement speaker for human { ... }"
/// let source = "τὸν λέκτην τῷ ἀνθρώπῳ ἐφαρμόζειν { }.";
/// let program = parse(source).unwrap();
///
/// if let Statement::TraitImpl(trait_impl) = &program.statements[0] {
///     assert_eq!(trait_impl.trait_name.normalized.as_str(), "λεκτην");
///     assert_eq!(trait_impl.type_name.normalized.as_str(), "ανθρωπω");
/// } else {
///     panic!("Expected TraitImpl");
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TraitImplDef {
    /// The specific archetype choosing to adopt these new behaviors.
    pub type_name: Word,
    /// The established interface or contract being fulfilled.
    pub trait_name: Word,
    /// The concrete actions outlining exactly how this archetype satisfies the trait's requirements.
    pub methods: Vec<ImplMethodDef>,
}

/// A concrete method implementation inside a [`TraitImplDef`]
///
/// This provides the actual sequence of statements (`body`) that execute
/// when a trait method is called on a specific type.
///
/// # Examples
///
/// ```rust,ignore
/// use glossa::parser::parse;
/// use glossa::ast::Statement;
///
/// let source = "τὸν λέκτην τῷ ἀνθρώπῳ ἐφαρμόζειν {
///     λέγειν { «Χαῖρε!» λέγε. }
/// }.";
/// let program = parse(source).unwrap();
///
/// if let Statement::TraitImpl(trait_impl) = &program.statements[0] {
///     let method = &trait_impl.methods[0];
///     assert_eq!(method.name.normalized.as_str(), "λεγειν");
///     assert_eq!(method.body.len(), 1);
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ImplMethodDef {
    /// The behavior from the trait that is being explicitly defined.
    pub name: Word,
    /// The runtime values provided during execution to carry out the action.
    pub params: Vec<FieldDecl>,
    /// The sequence of operations required to fulfill the method's purpose.
    pub body: Vec<Statement>,
}

/// A test block declaration (`δοκιμή`)
///
/// Tests in ΓΛΩΣΣΑ are first-class constructs. Users declare them using the `δοκιμή`
/// (test/trial) keyword, providing a string description and a block of statements.
/// The built-in testing framework automatically discovers and executes these during
/// `glossa test`.
///
/// # Examples
///
/// ```rust,ignore
/// use glossa::parser::parse;
/// use glossa::ast::Statement;
///
/// // Parse a test block
/// let source = "δοκιμή «πρόσθεσις» { ξ 1 ἔστω. }.";
/// let program = parse(source).unwrap();
///
/// if let Statement::TestDeclaration(test_decl) = &program.statements[0] {
///     assert_eq!(test_decl.name, "πρόσθεσις");
///     assert_eq!(test_decl.body.len(), 1);
/// } else {
///     panic!("Expected TestDeclaration");
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TestDecl {
    /// Test name (string literal)
    pub name: String,
    /// Test body statements
    pub body: Vec<Statement>,
}

/// A grammatical clause within a regular statement
///
/// Statements in ΓΛΩΣΣΑ can be complex and divided into multiple clauses,
/// separated by commas `,`. Each clause is further composed of expressions,
/// often chained together using the middle dot `·`.
///
/// This structure allows for the formulation of complex logical structures like
/// conditionals (`εἰ ..., τότε ...`), where each branch of the logic lives
/// in its own clause.
///
/// # Examples
///
/// ```rust,ignore
/// use glossa::parser::parse;
/// use glossa::ast::Statement;
///
/// // Parse a statement with two clauses separated by a comma
/// let source = "εἰ ἀληθές, «ναί» λέγε."; // "If true, say yes."
/// let program = parse(source).unwrap();
///
/// if let Statement::Regular { clauses, .. } = &program.statements[0] {
///     assert_eq!(clauses.len(), 2);
///     assert_eq!(clauses[0].expressions.len(), 2); // 'εἰ' and 'ἀληθές'
///     assert_eq!(clauses[1].expressions.len(), 2); // '«ναί»' and 'λέγε'
/// } else {
///     panic!("Expected Regular Statement");
/// }
/// ```
#[derive(Clone, PartialEq)]
pub struct Clause {
    /// Expressions in this clause (chained with middle dot)
    pub expressions: Vec<Expr>,
}

impl std::fmt::Debug for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            f.debug_struct("Clause")
                .field("expressions", &self.expressions)
                .finish()
        })
    }
}

impl std::fmt::Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            Statement::Regular {
                clauses,
                is_query,
                is_propagate,
            } => f
                .debug_struct("Regular")
                .field("clauses", clauses)
                .field("is_query", is_query)
                .field("is_propagate", is_propagate)
                .finish(),
            Statement::TypeDefinition(td) => f.debug_tuple("TypeDefinition").field(td).finish(),
            Statement::TraitDefinition(td) => f.debug_tuple("TraitDefinition").field(td).finish(),
            Statement::TraitImpl(ti) => f.debug_tuple("TraitImpl").field(ti).finish(),
            Statement::TestDeclaration(td) => f.debug_tuple("TestDeclaration").field(td).finish(),
        })
    }
}

impl Statement {
    /// Check if this statement ends with a question mark (`;` in Greek).
    ///
    /// In ΓΛΩΣΣΑ, appending the Greek question mark (U+037E) to a statement converts it
    /// into a query. This is commonly used during the REPL or for debugging to inspect
    /// the value of variables or expressions (e.g., `ξ;`).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use glossa::parser::parse;
    ///
    /// let stmt_regular = &parse("ξ ἔστω.").unwrap().statements[0];
    /// assert_eq!(stmt_regular.is_query(), false);
    ///
    /// let stmt_query = &parse("ξ;").unwrap().statements[0];
    /// assert_eq!(stmt_query.is_query(), true);
    /// ```
    pub fn is_query(&self) -> bool {
        match self {
            Statement::Regular { is_query, .. } => *is_query,
            Statement::TypeDefinition(_) => false,
            Statement::TraitDefinition(_) => false,
            Statement::TraitImpl(_) => false,
            Statement::TestDeclaration(_) => false,
        }
    }

    /// Check if this statement propagates errors (ends with `!`).
    ///
    /// Like the `?` operator in Rust, statements ending with `!` in ΓΛΩΣΣΑ
    /// attempt to evaluate an expression. If it is an error, they immediately
    /// return the error upward.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use glossa::parser::parse;
    ///
    /// let stmt_regular = &parse("ξ εὑρίσκειν.").unwrap().statements[0];
    /// assert_eq!(stmt_regular.is_propagate(), false);
    ///
    /// let stmt_propagate = &parse("ξ εὑρίσκειν!").unwrap().statements[0];
    /// assert_eq!(stmt_propagate.is_propagate(), true);
    /// ```
    pub fn is_propagate(&self) -> bool {
        match self {
            Statement::Regular { is_propagate, .. } => *is_propagate,
            Statement::TypeDefinition(_) => false,
            Statement::TraitDefinition(_) => false,
            Statement::TraitImpl(_) => false,
            Statement::TestDeclaration(_) => false,
        }
    }

    /// Retrieve the constituent clauses of this statement.
    ///
    /// This method safely extracts the inner `Clause` sequence for `Regular` statements.
    /// If the statement is a definition block (Type, Trait, etc.), it returns an empty slice,
    /// abstracting away the underlying enum variant matching.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use glossa::parser::parse;
    ///
    /// let stmt = &parse("εἰ ἀληθές, «ναί» λέγε.").unwrap().statements[0];
    /// assert_eq!(stmt.clauses().len(), 2);
    /// ```
    pub fn clauses(&self) -> &[Clause] {
        match self {
            Statement::Regular { clauses, .. } => clauses,
            Statement::TypeDefinition(_) => &[],
            Statement::TraitDefinition(_) => &[],
            Statement::TraitImpl(_) => &[],
            Statement::TestDeclaration(_) => &[],
        }
    }
}

/// An expression in GLOSSA
///
/// Expressions represent values that can be evaluated.
/// They include literals, variable references, operations, and function calls.
#[derive(Clone, PartialEq)]
pub enum Expr {
    /// A string literal: «text»
    ///
    /// # Example
    /// `«χαῖρε κόσμε»`
    StringLiteral(String),

    /// A number literal
    ///
    /// # Example
    /// `42` or `πέντε`
    NumberLiteral(i64),

    /// A boolean literal: ἀληθές or ψεῦδος
    ///
    /// # Example
    /// `ἀληθές` (True), `ψεῦδος` (False)
    BooleanLiteral(bool),

    /// An array literal: `[1, 2, 3]`
    ///
    /// # Example
    /// `[1, 2, 3]`
    ArrayLiteral(Vec<Expr>),

    /// Index access: `array[index]`
    ///
    /// # Example
    /// `πίναξ[0]`
    IndexAccess {
        /// The contiguous sequence of elements stored in memory from which an item is being retrieved.
        array: Box<Expr>,
        /// The numerical offset used to locate a specific element within the sequence.
        index: Box<Expr>,
    },

    /// A single Greek word with morphological information
    ///
    /// This is the most basic building block. The semantic analyzer uses
    /// the morphological information to determine if this word is a
    /// variable, a keyword, or part of a larger phrase.
    ///
    /// Unlike [`Phrase`](Expr::Phrase), a `Word` has been parsed as a single unit.
    Word(Word),

    /// Multiple terms forming a phrase
    ///
    /// Phrases are sequences of words that haven't been fully parsed into
    /// specific grammatical structures yet (e.g. parenthesized expressions).
    ///
    /// # Example
    /// `(ὁ ἄνθρωπος)`
    Phrase(Vec<Expr>),

    /// A property access (genitive construction)
    ///
    /// # Example
    /// `χρήστου ὄνομα` (the user's name)
    PropertyAccess {
        /// The complex structure from which a specific attribute is being extracted.
        owner: Box<Expr>,
        /// The specific named attribute being requested from the owning structure.
        property: Box<Expr>,
    },

    /// A function/verb call
    ///
    /// # Example
    /// `λέγε(«χαῖρε»)` (Explicit call syntax)
    Call {
        /// The action to perform, tied to an executable block of logic in the compiled program.
        verb: Word,
        /// The inputs supplied to fuel the execution of the requested action.
        arguments: Vec<Expr>,
    },

    /// Variable binding (ἔστω construction)
    ///
    /// # Example
    /// `ξ πέντε ἔστω` -> `Binding { name: "ξ", value: 5 }`
    Binding {
        /// The identifier introduced into the current lexical environment to refer to the stored data.
        name: Word,
        /// The computed result that is assigned to the newly introduced identifier.
        value: Box<Expr>,
    },

    /// Binary operation (arithmetic, comparison, boolean)
    ///
    /// # Example
    /// `x μεῖζον y` -> `x > y`
    BinOp {
        /// The first value partaking in the binary evaluation.
        left: Box<Expr>,
        /// The mathematical or logical comparison to execute between the two operands.
        op: BinOperator,
        /// The second value partaking in the binary evaluation.
        right: Box<Expr>,
    },

    /// Unary operation (negation)
    ///
    /// # Example
    /// `οὐκ x` -> `!x`
    UnaryOp {
        /// The logical inversion or singular operation to perform.
        op: UnaryOperator,
        /// The single value undergoing the operation.
        operand: Box<Expr>,
    },

    /// A block of statements in braces { ... }
    Block(Vec<Statement>),
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            Expr::StringLiteral(s) => f.debug_tuple("StringLiteral").field(s).finish(),
            Expr::NumberLiteral(n) => f.debug_tuple("NumberLiteral").field(n).finish(),
            Expr::BooleanLiteral(b) => f.debug_tuple("BooleanLiteral").field(b).finish(),
            Expr::ArrayLiteral(v) => f.debug_tuple("ArrayLiteral").field(v).finish(),
            Expr::IndexAccess { array, index } => f
                .debug_struct("IndexAccess")
                .field("array", array)
                .field("index", index)
                .finish(),
            Expr::Word(w) => f.debug_tuple("Word").field(w).finish(),
            Expr::Phrase(v) => f.debug_tuple("Phrase").field(v).finish(),
            Expr::PropertyAccess { owner, property } => f
                .debug_struct("PropertyAccess")
                .field("owner", owner)
                .field("property", property)
                .finish(),
            Expr::Call { verb, arguments } => f
                .debug_struct("Call")
                .field("verb", verb)
                .field("arguments", arguments)
                .finish(),
            Expr::Binding { name, value } => f
                .debug_struct("Binding")
                .field("name", name)
                .field("value", value)
                .finish(),
            Expr::BinOp { left, op, right } => f
                .debug_struct("BinOp")
                .field("left", left)
                .field("op", op)
                .field("right", right)
                .finish(),
            Expr::UnaryOp { op, operand } => f
                .debug_struct("UnaryOp")
                .field("op", op)
                .field("operand", operand)
                .finish(),
            Expr::Block(stmts) => f.debug_tuple("Block").field(stmts).finish(),
        })
    }
}

/// Binary operators in GLOSSA
///
/// # Why it exists
///
/// This enum represents the various mathematical and logical operations that can
/// be performed between two expressions. It translates Ancient Greek concepts
/// like `καί` (and), `ἤ` (or), and `μεῖζον` (greater) into abstract semantic nodes.
///
/// ## Examples
///
/// ```rust,ignore
/// use glossa::ast::BinOperator;
///
/// let addition = BinOperator::Add;
/// let and_op = BinOperator::And;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOperator {
    // Arithmetic
    /// Addition (sum) - `ἄθροισμα`
    Add,
    /// Subtraction (difference) - `διαφορά`
    Sub,
    /// Multiplication (product) - `γινόμενον`
    Mul,
    /// Division (quotient/part) - `μέρος`
    Div,
    /// Modulo (remainder) - `ὑπόλοιπον`
    Mod,

    // Comparison
    /// Equality - `ἴσον`
    Eq,
    /// Inequality - `ἄνισον`
    Ne,
    /// Less than - `ἔλαττον`
    Lt,
    /// Less than or equal - `ἔλαττον ἢ ἴσον`
    Le,
    /// Greater than - `μεῖζον`
    Gt,
    /// Greater than or equal - `μεῖζον ἢ ἴσον`
    Ge,

    // Boolean
    /// Logical AND - `καί`
    And,
    /// Logical OR - `ἤ`
    Or,
}

/// Unary operators in GLOSSA
///
/// # Why it exists
///
/// This enum represents logical inversions or operations that apply to a single
/// operand, such as logical negation (`οὐκ`, "not").
///
/// ## Examples
///
/// ```rust,ignore
/// use glossa::ast::UnaryOperator;
///
/// let not_op = UnaryOperator::Not;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    /// Logical Negation - `οὐ`, `οὐκ`, `οὐχ`
    Not,
    /// Arithmetic Negation
    Neg,
    /// Unwrap (Extract value) - `!` suffix
    Unwrap,
}

/// Represents a parsed word from the ΓΛΩΣΣΑ source code.
///
/// A `Word` in the AST is not just a `String`. Because Ancient Greek has
/// rich diacritics (accents, breathings) and capitalization, comparing words
/// directly is error-prone. This struct acts as a bridge between the
/// human-readable source and the compiler's internal matching logic,
/// preserving the original polytonic text (for display) while providing
/// a normalized version for compiler analysis.
///
/// ## Why it exists
///
/// In the morphological analysis phase, the compiler needs to look up words
/// in its lexicon. If a user writes `Ἄνθρωπος` (Capital A, smooth breathing, acute accent),
/// the compiler must recognize it as the same lemma as `ἄνθρωπος` (lowercase a, smooth, acute)
/// or even `ανθρωπος` (stripped of all diacritics).
///
/// Storing both the `original` string (for exact error reporting) and the `normalized`
/// string (for lexicon lookups and comparisons) in a single struct solves this problem
/// at the parsing stage, preventing expensive re-allocations later.
///
/// ## Examples
///
/// You can construct a `Word` via `Word::new` for convenience, or construct it directly
/// using [`smol_str::SmolStr`] to avoid heap allocations for small strings.
///
/// ```rust,ignore
/// use glossa::ast::Word;
///
/// let word = Word::new("Ἀθῆναι");
/// assert_eq!(word.original.as_str(), "Ἀθῆναι");
/// assert_eq!(word.normalized.as_str(), "αθηναι");
/// ```
///
/// ```rust,ignore
/// use glossa::ast::Word;
/// use smol_str::SmolStr;
///
/// // Create a word representing the noun "ἄνθρωπος" (man) directly
/// let man = Word {
///     original: SmolStr::new("Ἄνθρωπος"),
///     normalized: SmolStr::new("ανθρωπος"),
/// };
///
/// assert_eq!(man.original.as_str(), "Ἄνθρωπος");
/// assert_eq!(man.normalized.as_str(), "ανθρωπος");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    /// The exact string from the source code, preserving case and diacritics.
    /// Useful for accurate compilation error messages and source maps.
    pub original: SmolStr,
    /// The lowercase, diacritic-stripped version of the word.
    /// Used for lexicon lookups, keyword matching, and semantic hashing.
    pub normalized: SmolStr,
}

impl Word {
    /// Creates a new `Word` and automatically pre-computes its normalized form.
    ///
    /// It is vital that Greek text is normalized (lowercase, diacritics stripped)
    /// as early as possible so that semantic comparisons (`Άνθρωπος` == `ἄνθρωπος`)
    /// are reliable throughout the compilation pipeline.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use glossa::ast::Word;
    ///
    /// let word = Word::new("Ἀθῆναι");
    /// assert_eq!(word.original.as_str(), "Ἀθῆναι");
    /// assert_eq!(word.normalized.as_str(), "αθηναι");
    /// ```
    pub fn new(original: impl Into<SmolStr>) -> Self {
        let original = original.into();
        let normalized = crate::text::normalize_greek(&original);
        Word {
            original,
            normalized,
        }
    }
}
