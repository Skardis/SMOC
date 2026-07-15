use smoc_latex::parse_latex;
use smoc_core::SmocEngine;
use smoc_arithmetics::{rule_factorize_add, rule_simplify_fraction, rule_add_numbers};
use smoc_algebra::{rule_multiply_powers, penalty_root_in_denominator};

fn run_test_case(name: &str, latex: &str) {
    println!("\n=== {} ===", name);
    println!("Vstup: {}", latex);
    
    // 1. Parsing
    let expr = match parse_latex(latex) {
        Ok(e) => e,
        Err(e) => {
            println!("❌ CHYBA PARSOVÁNÍ: {}", e);
            return;
        }
    };
    
    println!("AST: {:?}", expr);

    // 2. Sestavení Enginu se všemi pravidly
    let mut engine = SmocEngine::new();
    engine.add_rule("Vytýkání ze sčítání", rule_factorize_add);
    engine.add_rule("Krácení zlomků", rule_simplify_fraction);
    engine.add_rule("Sčítání čísel", rule_add_numbers);
    engine.add_rule("Sčítání exponentů", rule_multiply_powers);
    engine.add_penalty_rule("Odmocnina ve jmenovateli", penalty_root_in_denominator);

    // 3. Výpočet
    let (final_expr, steps) = engine.simplify_with_steps(&expr);
    
    println!("--- Postup řešení ---");
    if steps.is_empty() {
        println!("(Žádné úpravy nebyly potřeba / Nenašlo se řešení)");
    } else {
        for (i, step) in steps.iter().enumerate() {
            println!("Krok {}: {}", i + 1, step);
        }
    }
    
    println!("Výsledek: {:?}", final_expr);
}

#[test]
fn run_all_user_tests() {
    // === ARITMETIKA ===
    run_test_case("Aritmetika - 1. Úroveň", "\\frac{8}{12}");
    run_test_case("Aritmetika - 2. Úroveň", "\\frac{45}{60}");
    run_test_case("Aritmetika - 3. Úroveň", "\\frac{2}{3} \\cdot \\frac{9}{8}");
    // Nahrazen znak ":" znakem zlomku, jelikož parser to zatím neumí (dělení je stejné jako zlomek)
    run_test_case("Aritmetika - 4. Úroveň", "\\frac{\\frac{14}{25}}{\\frac{21}{10}}");
    run_test_case("Aritmetika - 5. Úroveň", "\\frac{120}{180}");
    run_test_case("Aritmetika - 6. Úroveň", "\\frac{2^3 \\cdot 3^2}{2 \\cdot 3^3}");
    run_test_case("Aritmetika - 7. Úroveň", "\\frac{15}{4} - \\frac{5}{6}");
    run_test_case("Aritmetika - 8. Úroveň", "\\frac{75 \\cdot 48}{40 \\cdot 90}");
    run_test_case("Aritmetika - 9. Úroveň", "\\frac{\\frac{3}{4} + \\frac{1}{2}}{\\frac{5}{6} - \\frac{1}{3}}");
    run_test_case("Aritmetika - 10. Úroveň", "\\frac{2^{10} \\cdot 15^3}{6^4 \\cdot 10^2}");

    // === ALGEBRA ===
    run_test_case("Algebra - 1. Úroveň", "\\frac{3x}{6x^2}");
    run_test_case("Algebra - 2. Úroveň", "\\frac{a^2 - a}{a}");
    run_test_case("Algebra - 3. Úroveň", "\\frac{4x + 8}{2}");
    run_test_case("Algebra - 4. Úroveň", "\\frac{x^2 - 9}{x + 3}");
    run_test_case("Algebra - 5. Úroveň", "\\frac{a^2 - 2ab + b^2}{a - b}");
    run_test_case("Algebra - 6. Úroveň", "\\frac{3x^2 - 3y^2}{6x + 6y}");
    run_test_case("Algebra - 7. Úroveň", "\\frac{x^2 - 5x + 6}{x - 2}");
    run_test_case("Algebra - 8. Úroveň", "\\frac{a^3 - b^3}{a^2 - b^2}");
    run_test_case("Algebra - 9. Úroveň", "\\frac{2x^2 + 5x - 3}{4x^2 - 1}");
    run_test_case("Algebra - 10. Úroveň", "\\frac{x^3 - x^2 - x + 1}{x^2 - 2x + 1}");
}
