# What is SMOC?

**Skardis Math On Crack (SMOC)** is a high-performance, symbolic mathematics engine written in Rust. 
Unlike a standard calculator that immediately converts all numbers to treacherous `float` data types (leading to floating point inaccuracies and rounding errors), SMOC treats all math as **Abstract Syntax Trees** (AST). 

A fraction like $\frac{10}{3}$ doesn't mean `3.33333333`. It is structurally preserved as `Fraction(10, 3)` until further manipulated algebraically. This guarantees 100% precision even for the most chaotic algebraic equations.

## Core Vision
1. **Extensibility:** Every mathematical formula in SMOC is just a "Plugin" (A Rule). Anyone can add a rule for derivatives or matrices, and the Engine will automatically know how to apply it.
2. **Precision (Absolute):** Integer fractions, decimal-to-fraction localized parsing, and strict isolation from float computations.
3. **Performance and Elegance:** Rust ensures extreme memory safety, blazing speed (thanks to our BFS state-space search engine) and elegant backend architecture.

## Why "On Crack"?
This engine stops at nothing. It will ingest brutal Latex syntax, lazily generate thousands of prime numbers using lightning-fast caches (`smoc_prime`), perform immense binomial expansions over brackets using Pascal's triangle (`smoc_pascaltriangle`) and spit out the simplified, reduced math before you can blink.
