# Echo's DX Audit Report 🗣️

## 1. Experience: The Walkthrough

I attempted to run the examples provided in the `README.md` and the `examples/` directory.

### ✅ Successes
*   **README Example (`hero.γλ`):** Ran successfully! Output: `Σωκράτης`.
*   **`examples/hello.γλ`:** Ran successfully! Output: `χαῖρε κόσμε`.
*   **`examples/variables.γλ`:** Ran successfully! Output: `5`.

The "Quick Start" experience was smooth. Copy-pasting the code worked as advertised.

## 2. Stumble: The Friction Points

I intentionally tried to break the code to test the error messages.

### ✅ Syntax Errors
*   **Action:** Removed the final period from `hero.γλ`.
*   **Result:** The compiler correctly identified a syntax error:
    ```
    Error:   × Σφάλμα συντάξεως: Parse error: ... expected statement_end
    ```
    The error message was localized to the correct line.

### ⚠️ Semantic Errors (Mixed Bag)

Here, things got weird.

#### ✅ Undefined Type
*   **Action:** Used an undefined type `ΜηΥπαρκτος` in `hero.γλ`.
*   **Result:** A clear, helpful error message:
    ```
    Error:   × Ἄγνωστον ὄνομα: μηυπαρκτος
    ```
    This is excellent!

#### ❌ Undefined Variable (Silent Failure / Defaulting)
*   **Action:** Assigned an undefined variable `β` to `α`: `α β ἔστω.` (See `repro_silent_default.γλ`)
*   **Result:** The program compiled and ran, printing `0`!
    ```
    0
    ```
    This is dangerous. A typo in a variable name could silently introduce bugs instead of failing at compile time.

#### ❌ Undefined Variable (Leaky Abstraction)
*   **Action:** Used an undefined variable in a different context: `α β foo.` (See `repro_crash.γλ`)
*   **Result:** The compiler crashed with a raw Rust error, exposing internal implementation details:
    ```
    Error:   ×  Rustc Error:
      │ error[E0425]: cannot find value `g__u3b1_` in this scope
    ```
    The user sees `g__u3b1_` instead of `α`. This is confusing and breaks the immersion of the language.

## 3. Recommendations

1.  **Fix Silent Defaulting:** Undefined variables should validly error, not default to `0`.
2.  **Hide Rust Errors:** The compiler should catch undefined variables during the semantic analysis phase, before generating Rust code, to prevent raw `rustc` errors from reaching the user.
3.  **Consistent Error Messages:** Ensure all semantic errors (like undefined variables) use the localized `Ἄγνωστον ὄνομα` format.
