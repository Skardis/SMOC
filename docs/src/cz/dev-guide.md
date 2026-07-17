# Průvodce pro Vývojáře (Developer Guide)

Chcete rozšířit SMOC o nové schopnosti, gramatiku nebo matematická pravidla? Skvělé! Architektura SMOC je navržena tak, aby byla plně modulární a oddělená do samostatných knihoven (Crates).

Zde je návod, jak vytvořit 3 typické moduly: Parser (`smoc_typst`), Systém pravidel (`smoc_geometry`) a Slovník (`smoc_pi`).

---

## 1. Tvorba nového Parseru (např. `smoc_typst`)
V současné chvíli používáme `smoc_latex`, který transformuje kód z LaTeXu do našeho Abstraktního Syntaktického Stromu (AST) ve `smoc_core::Expr`. Pokud byste chtěli napsat podporu pro např. značkovací jazyk [Typst](https://typst.app/), vytvoříte novou knihovnu.

**Kroky:**
1. Ve složce workspace zavolejte `cargo new --lib smoc_typst`.
2. Do `Cargo.toml` vložte závislosti na `smoc_core` a knihovnu na parsování (např. `pest`).
3. Napište gramatiku (např. `typst.pest`), která rozeznává zlomky ve formátu `x / y`.
4. Vytvořte funkci např. `pub fn parse_typst(input: &str) -> Expr`, která vygeneruje AST strom struktury `Expr::Node("Fraction", vec![x, y])`.

*Tip: Engine nezajímá, jak k němu strom došel. Stačí ho do enginu poslat a ten už ho vyřeší.*

---

## 2. Tvorba Pravidlové sady (např. `smoc_geometry`)
Máte zájem, aby SMOC uměl pracovat se siny, kosiny nebo obsahem kruhu? Založte modul s pravidly, podobně jako máme `smoc_arithmetics` nebo `smoc_algebra`.

**Kroky:**
1. Vytvořte `cargo new --lib smoc_geometry`.
2. Naimplementujte funkci s předepsanou hlavičkou pro Engine: `pub fn rule_name(expr: &Expr) -> Option<Expr>`.
3. Příklad pravidla "Pythagorova věta": Pokud vám do pravidla přijde strom tvaru $sin^2(x) + cos^2(x)$, vaše funkce pravidla vrátí `Some(Expr::Number(1))`. Ve všech ostatních případech vrátí `None`.
4. V cílové aplikaci jednoduše přidáte pravidlo do stroje: `engine.add_rule("Pythagorova věta", smoc_geometry::rule_pythagoras);`.

Tímto se z vaší geometrie stává plnohodnotný plugin v celém SMOC ekosystému a Engine ho začne automaticky využívat.

---

## 3. Tvorba Rychlých Slovníků (např. `smoc_pi`)
SMOC potřebuje pro některé výpočty obrovské lookup tabulky (jako např. prvočísla v `smoc_prime` nebo koeficienty v `smoc_pascaltriangle`). Můžete vytvořit modul např. pro analýzu čísla Pi.

**Kroky:**
1. Vytvořte `cargo new --lib smoc_pi`.
2. Vytvořte strukturu, která drží obří sadu dat v paměti a využívá `OnceLock<Mutex<VaseStruktura>>` pro zajištění bezpečnosti při více vláknech (Concurrency).
3. Vytvořte konfiguraci `PiConfig`, která umožňuje uživateli určit např. max_limit. (Nikdy nechcete, aby váš slovník omylem začal generovat miliardu číslic Pí do paměti RAM a shodil uživateli počítač).
4. Připravte funkce pro ukládání a načítání vygenerovaného slovníku na disk (např. pomocí knihovny `bincode`), aby to Engine nemusel počítat pokaždé znovu!
