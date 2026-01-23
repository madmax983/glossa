# 20. Complete Grammar

```ebnf
(* Top Level *)
program        = { statement }
statement      = expression_list "."
               | expression_list ";"

expression_list = expression { "·" expression }

(* Expressions *)
expression     = term+

term           = literal
               | greek_word
               | "[" [expression {"," expression}] "]"
               | "(" expression ")"

literal        = string_literal
               | number_literal
               | boolean_literal

string_literal = "«" {character} "»"
               | '"' {character} '"'

number_literal = digit+
               | digit+ "." digit+
               | "0x" hex_digit+
               | "0b" binary_digit+
               | greek_numeral

boolean_literal = "ἀληθές" | "ψεῦδος"

greek_word     = greek_char+

greek_char     = greek_letter | combining_mark

(* Comments *)
comment        = "//" {any} newline
               | "/*" {any} "*/"
```