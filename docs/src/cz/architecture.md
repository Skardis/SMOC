# Architektura a AI Engine

Jádrem SMOC není jen sbírka algoritmů, ale **chytrý prohledávač stavového prostoru** (State-Space Search Engine) napsaný ve `smoc_core/src/lib.rs`.

Pojďme se podívat pod pokličku, jak tohle "matematické AI" funguje.

## 1. Abstraktní Syntaktický Strom (AST)
Každá rovnice, ať je jakkoliv složitá, je reprezentována stromem `Expr`:
```rust
pub enum Expr {
    Number(i64),
    Variable(String),
    Node(String, Vec<Expr>), // např. Node("Add", [Number(5), Variable("x")])
}
```

## 2. Bodovací systém (LeafCount)
Každý výraz má své "skóre" (složitost). Čím méně uzlů a znaků strom obsahuje, tím menší skóre získá. Cílem AI je minimalizovat toto skóre na nejnižší možnou úroveň. Pokud je zavedeno nějaké penalizační pravidlo (např. *Odmocnina nesmí být ve jmenovateli*), Engine k danému stavu připočte obrovské negativní body.

## 3. Paralelní Vesmíry (Breadth-First Search)
Jak SMOC řeší matematiku:
1. Přečte AST a zkusí aplikovat **VŠECHNA** dostupná pravidla (např. Krácení zlomků, Sčítání exponentů).
2. Tím vytvoří nové paralelní "vesmíry" (stavy rovnice). Všechny tyto vesmíry zařadí do fronty (`VecDeque`).
3. Z fronty vytahuje postupně stavy a aplikuje na ně další pravidla.
4. **Paměť (HashSet):** Pokaždé si pamatuje strom, který už viděl. Pokud se nějakou oklikou dostane ke stejnému výrazu (např. přes přičítání nuly), okamžitě tuto větev zahodí, aby zabránil zacyklení.

## 4. Ochrany Enginu
Engine obsahuje brzdy, aby zamezil zavaření procesoru:
- **Záchranná brzda:** Po `2000` krocích vyhodnocování se násilně zastaví a vrátí dosavadní nejlepší výsledek.
- **Early Stop:** Narazí-li na výraz se složitostí 1 (např. `x` nebo `42`), okamžitě ví, že lépe už to zkrátit nejde, a algoritmus s obřím jásotem končí!

Díky této logice SMOC sám odmítá nesmyslné expanze. Například rozvoj `(x+y)^3` na obří mnohočlen normálně odmítne, protože by to **zvětšilo** složitost. Zvolí si ho jen tehdy, pokud tato expanze umožní následné pokrácení a likvidaci jiných členů ve zlomku, čímž *sníží* celkové konečné skóre. To je síla SMOC!
