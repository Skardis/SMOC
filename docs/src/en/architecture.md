# Architecture and AI Engine

At its core, SMOC isn't just a collection of hardcoded algorithms. It is a **State-Space Search Engine** residing in `smoc_core/src/lib.rs`.

Let's look under the hood to see how this mathematical AI actually works.

## 1. Abstract Syntax Tree (AST)
Every equation, no matter how complex, is represented as an `Expr` tree:
```rust
pub enum Expr {
    Number(i64),
    Variable(String),
    Node(String, Vec<Expr>), // e.g., Node("Add", [Number(5), Variable("x")])
}
```

## 2. Scoring System (LeafCount)
Every expression has a "score" (complexity). The fewer nodes and characters a tree has, the lower its score. The Engine's goal is to minimize this score to the absolute lowest possible level. If a Penalty Rule is triggered (e.g., *No roots in the denominator*), the Engine slaps massive negative points to that state.

## 3. Parallel Universes (Breadth-First Search)
How SMOC solves math:
1. It reads the AST and tries to apply **ALL** available rules (e.g., Fraction simplification, Exponent addition).
2. This creates new parallel "universes" (equation states). All these universes are pushed into a queue (`VecDeque`).
3. It pops states from the queue one by one and applies further rules on them.
4. **Memory (HashSet):** Every time it encounters a state, it remembers it. If it loops back to an identical tree (for example, by adding zero), it instantly drops that branch to prevent infinite loops.

## 4. Engine Safeguards
The engine is equipped with brakes to prevent CPU meltdowns:
- **Emergency Brake:** After `2000` steps of search space exploration, it forcefully halts and returns the best result so far.
- **Early Stop:** If it hits an expression with a complexity of 1 (e.g., `x` or `42`), it instantly knows math doesn't get simpler than that, and it triumphantly ends the algorithm early!

Because of this logic, SMOC naturally rejects useless expansions. For instance, expanding `(x+y)^3` into a giant polynomial is normally rejected because it **increases** complexity. It only chooses to follow that expansion path if doing so allows for subsequent cancellations in a fraction, thereby *decreasing* the final overall score. This is the ultimate power of SMOC!
