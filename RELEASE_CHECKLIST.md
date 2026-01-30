# GLOSSA v0.1.0 Release Checklist

## ✅ Core Features (COMPLETE)

- [x] **Compiler Pipeline** - Lexer → Parser → Semantic Analysis → IR → Rust Codegen
- [x] **Type System** - User-defined structs, trait definitions, trait impls
- [x] **Control Flow** - Conditionals (εἰ), loops (ἕως, διά), pattern matching (κατά)
- [x] **Functions** - First-class functions with Greek verb syntax
- [x] **Collections** - Arrays, HashMap (χάρτης), HashSet (σύνολον)
- [x] **Mutability** - Mutable bindings (μετά) and assignment (γίγνεται)
- [x] **Lambda Expressions** - Participles as closures (present/aorist/perfect)
- [x] **Iterator Operations** - map, filter, find, fold, any, all
- [x] **Option/Result** - Rust-style error handling with Greek syntax
- [x] **Operators** - Arithmetic, comparison, logical
- [x] **String Operations** - Contains, split, join

## ✅ Quality (STRONG)

- [x] **Test Coverage** - 294/294 tests passing (100%)
- [x] **CI Pipeline** - GitHub Actions with codecov integration
- [x] **Error Messages** - Greek error messages with miette diagnostics
- [x] **Documentation** - 22 reference docs covering all features
- [x] **Examples** - hello.γλ, variables.γλ, collections_demo.γλ, leetcode_working.γλ

## ⚠️ Known Issues

- [ ] **Diacritic Normalization** - Issue #93: ἓν vs ἐν collision
  - Workaround: Use numeric literals (0, 1, 5) instead of Greek number words
  - Should be fixed before v1.0, acceptable for v0.1

## 🚧 Pre-Release Tasks

### Critical (Must Have)
- [ ] **LICENSE** - Add license file (which license?)
- [ ] **CHANGELOG.md** - Document v0.1.0 features and breaking changes
- [ ] **Installation docs** - Add cargo install instructions to README

### Important (Should Have)
- [ ] **CONTRIBUTING.md** - Contribution guidelines
- [ ] **Release notes** - GitHub release with highlights
- [ ] **Cargo.toml metadata** - Add repository, homepage, keywords, categories
- [ ] **README polish** - Add installation, quick start, feature showcase

### Nice to Have
- [ ] **Website/landing page** - docs.rs or GitHub Pages
- [ ] **Tutorial** - Step-by-step guide for newcomers
- [ ] **Video demo** - Screencast showing GLOSSA in action
- [ ] **Benchmarks** - Performance comparison vs hand-written Rust

## 📦 Release Artifacts

### For GitHub Release
- [ ] Source tarball (auto-generated)
- [ ] Compiled binaries for:
  - [ ] Linux (x86_64)
  - [ ] macOS (x86_64, aarch64)
  - [ ] Windows (x86_64)

### For crates.io (Optional for v0.1)
- [ ] Publish to crates.io
- [ ] Verify docs.rs build

## 🎯 Release Timeline

**Status:** Close! Missing mostly project hygiene files.

**Minimum viable release (v0.1.0-alpha):**
1. Add LICENSE (1 hour)
2. Add CHANGELOG.md (1 hour)
3. Update README with installation (1 hour)
4. Tag and push (30 min)

**Estimated time to minimal release:** ~4 hours

**Full v0.1.0 release (with binaries, crates.io):**
- Add all "Important" items above
- Set up binary builds in CI
- Estimated time: 1-2 days

## 💭 Post-Release Roadmap

### v0.2.0
- [ ] Fix diacritic normalization (#93)
- [ ] Add standard library (I/O, filesystem, etc.)
- [ ] Module system

### v0.5.0
- [ ] Package manager (cargo-like)
- [ ] LSP server for editor support
- [ ] Formatter (gfmt)

### v1.0.0
- [ ] Stable language spec
- [ ] Production-ready compiler
- [ ] Comprehensive standard library

---

**Recommendation:** Ship v0.1.0-alpha with just LICENSE + CHANGELOG + README updates. This gets GLOSSA into users' hands quickly while being transparent about alpha status.
