# 20. Complete Grammar

This EBNF reflects the actual PEG grammar (`glossa.pest`).

```ebnf
(* ============ Top Level ============ *)
program           = { statement }

statement         = test_declaration
                    | ( type_definition
                      | trait_definition
                      | trait_impl
                      | clause_list )
                      statement_end

statement_end     = period | query | propagate

(* ============ Definitions ============ *)

(* εἶδος typename ὁρίζειν { fields } *)
type_definition   = eidos_keyword greek_word orizontin_keyword
                    "{" [ field_list ] "}"

(* χαρακτήρ traitname ὁρίζειν { methods } *)
trait_definition   = character_keyword greek_word orizontin_keyword
                    "{" [ trait_method_list ] "}"

(* εἶδος typename τῷ traitname ἐμπίπτειν { methods } *)
trait_impl         = eidos_keyword greek_word dative_marker greek_word
                    empiptein_keyword "{" [ impl_method_list ] "}"

(* δοκιμή «test name». body τέλος. *)
test_declaration   = ("δοκιμή" | "δοκιμη") string_literal statement_end
                    test_body
                    ("τέλος" | "τελος") statement_end

test_body          = { !(("τέλος" | "τελος")) statement }

(* ============ Keywords ============ *)
eidos_keyword      = "εἶδος" | "ειδος"
character_keyword  = "χαρακτήρ" | "χαρακτηρ"
orizontin_keyword  = "ὁρίζειν" | "οριζειν"
dative_marker      = "τῷ" | "τω"
empiptein_keyword  = "ἐμπίπτειν" | "εμπιπτειν"
dei_keyword        = "δεῖ" | "δει"
ede_keyword        = "ἤδη" | "ηδη"
dokime_keyword     = "δοκιμή" | "δοκιμη"
telos_keyword      = "τέλος" | "τελος"

(* ============ Fields & Methods ============ *)
field_list         = field_declaration { (chain | period) field_declaration }
                    [ chain | period ]

field_declaration  = greek_word greek_word

trait_method_list  = trait_method { period trait_method } [ period ]

(* δεῖ/ἤδη method params [· body] *)
trait_method       = (dei_keyword | ede_keyword) greek_word+
                    [ chain statement+ ]

impl_method_list   = impl_method+

(* method params · body_statement *)
impl_method        = greek_word+ chain statement

(* ============ Clauses ============ *)
clause_list       = clause { comma clause }

clause            = expression { chain expression }

(* ============ Expressions ============ *)
expression        = term+

term              = block
                  | array_literal
                  | string_literal
                  | number_literal
                  | boolean_literal
                  | unwrap_expr
                  | indexed_word
                  | parenthesized_expr
                  | greek_word

unwrap_expr       = greek_word "!"

parenthesized_expr = "(" expression ")"

indexed_word      = greek_word "[" index_expr "]"
index_expr        = number_literal | greek_word

array_literal     = "[" [ array_elements ] "]"
array_elements    = array_element { "," array_element }
array_element     = string_literal | number_literal
                  | boolean_literal | greek_word

block             = "{" statement* "}"

(* ============ Literals ============ *)
string_literal    = "«" { character } "»"

number_literal    = digit+

boolean_literal   = "ἀληθές" | "ψεῦδος" | "αληθες" | "ψευδος"

(* ============ Words ============ *)
greek_word        = greek_char+
                  | ascii_alpha { ascii_alphanumeric | "_" }

greek_char        = greek_letter | greek_combining

greek_letter      = U+0370..U+037D | U+037F..U+0386
                  | U+0388..U+03FF | U+1F00..U+1FFF

greek_combining   = U+0300..U+036F

(* ============ Punctuation ============ *)
period            = "."                  (* τελεία — statement end *)
chain             = U+00B7              (* ἄνω τελεία — expression chain *)
query             = U+037E | "?"        (* ἐρωτηματικό — query *)
propagate         = ";"                  (* propagation — converts to ? *)
comma             = ","                  (* clause separator *)

(* ============ Comments ============ *)
comment           = "//" { any } newline
```