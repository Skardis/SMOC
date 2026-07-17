# Developer Guide

Do you want to extend SMOC with new parsing capabilities, grammars, or mathematical rules? Excellent! SMOC's architecture is fully modular and separated into independent crates.

Here is a guide on how to build the 3 typical modules: A Parser (`smoc_typst`), a Rule System (`smoc_geometry`), and a Dictionary (`smoc_pi`).

---

## 1. Creating a New Parser (e.g., `smoc_typst`)
Currently, we use `smoc_latex` to transform LaTeX syntax into our Abstract Syntax Tree (AST) defined in `smoc_core::Expr`. If you wish to build support for a markup language like [Typst](https://typst.app/), you will create a new crate.

**Steps:**
1. In the workspace folder, run `cargo new --lib smoc_typst`.
2. In `Cargo.toml`, add dependencies for `smoc_core` and a parsing crate (e.g., `pest`).
3. Write a grammar (e.g., `typst.pest`) that recognizes fractions like `x / y`.
4. Create a function like `pub fn parse_typst(input: &str) -> Expr` that generates the AST tree `Expr::Node("Fraction", vec![x, y])`.

*Tip: The Engine doesn't care how the tree was built. Just feed it the tree, and it will solve it.*

---

## 2. Creating a Rule Set (e.g., `smoc_geometry`)
Interested in having SMOC work with sines, cosines, or circle areas? Set up a rule module, exactly like `smoc_arithmetics` or `smoc_algebra`.

**Steps:**
1. Create `cargo new --lib smoc_geometry`.
2. Implement a function with the required Engine signature: `pub fn rule_name(expr: &Expr) -> Option<Expr>`.
3. Example rule "Pythagorean Theorem": If the rule receives a tree of the form $sin^2(x) + cos^2(x)$, your rule returns `Some(Expr::Number(1))`. In all other cases, it returns `None`.
4. In the target application, simply inject the rule into the machine: `engine.add_rule("Pythagorean Theorem", smoc_geometry::rule_pythagoras);`.

With that single line, your geometry module becomes a full-fledged plugin in the SMOC ecosystem, and the Engine will start utilizing it automatically.

---

## 3. Creating Lightning-Fast Dictionaries (e.g., `smoc_pi`)
SMOC requires massive lookup tables for certain computations (like primes in `smoc_prime` or coefficients in `smoc_pascaltriangle`). You can create a module for analyzing Pi digits.

**Steps:**
1. Create `cargo new --lib smoc_pi`.
2. Create a struct that holds the massive dataset in memory. Wrap it in a `OnceLock<Mutex<YourStruct>>` to ensure Thread-Safe Concurrency.
3. Build a configuration `PiConfig` that allows users to define parameters like `max_limit`. (You never want your dictionary to accidentally generate a billion digits of Pi into RAM and crash the user's PC).
4. Prepare functions for saving and loading the generated dictionary to disk (e.g., using `bincode`), so the Engine doesn't have to recalculate it every time!
