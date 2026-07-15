use smoc_core::Expr;
use std::collections::HashMap;

// Pravidlo: x^a * x^b = x^{a+b}
// Umí spojit i "x * x^2" (protože x chápe jako x^1)
pub fn rule_multiply_powers(expr: &Expr) -> Option<Expr> {
    if let Expr::Node(name, items) = expr {
        if name == "Multiply" {
            let mut bases: HashMap<Expr, Vec<Expr>> = HashMap::new();
            let mut other_items = Vec::new();
            let mut changed = false;

            for item in items {
                match item {
                    Expr::Node(n, children) if n == "Power" && children.len() == 2 => {
                        let base = children[0].clone();
                        let exp = children[1].clone();
                        bases.entry(base).or_insert_with(Vec::new).push(exp);
                    },
                    Expr::Variable(_) => {
                        bases.entry(item.clone()).or_insert_with(Vec::new).push(Expr::Number(1));
                    },
                    _ => {
                        other_items.push(item.clone());
                    }
                }
            }
            
            let mut final_items = other_items;
            for (base, exps) in bases {
                if exps.len() > 1 {
                    changed = true; // Našli jsme stejné základy, můžeme je sloučit!
                    let combined_exp = Expr::Node("Add".to_string(), exps);
                    final_items.push(Expr::Node("Power".to_string(), vec![base, combined_exp]));
                } else {
                    let exp = exps[0].clone();
                    if exp == Expr::Number(1) {
                        final_items.push(base); // vrátíme zpět obyčejné 'x'
                    } else {
                        final_items.push(Expr::Node("Power".to_string(), vec![base, exp]));
                    }
                }
            }

            if changed {
                if final_items.len() == 1 {
                    return Some(final_items.pop().unwrap());
                } else {
                    return Some(Expr::Node("Multiply".to_string(), final_items));
                }
            }
        }
    }
    None
}

// Penaltové pravidlo: Odmocnina nesmí být ve jmenovateli!
pub fn penalty_root_in_denominator(expr: &Expr) -> usize {
    if let Expr::Node(name, items) = expr {
        if name == "Fraction" && items.len() == 2 {
            // Funkce pro hledání zlomkového exponentu (odmocniny) uvnitř jakéhokoliv stromu
            fn contains_fractional_power(e: &Expr) -> bool {
                match e {
                    Expr::Node(n, children) if n == "Power" && children.len() == 2 => {
                        if let Expr::Node(n2, _) = &children[1] {
                            if n2 == "Fraction" { return true; }
                        }
                        false
                    },
                    Expr::Node(_, children) => {
                        children.iter().any(|c| contains_fractional_power(c))
                    },
                    _ => false,
                }
            }
            
            // Kontrola jmenovatele
            if contains_fractional_power(&items[1]) {
                return 1000; // Krutá penalizace, Engine se této cestě vyhne!
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use smoc_core::SmocEngine;

    #[test]
    fn test_multiply_powers() {
        // x * x^2 => x^(1+2)
        let math_expr = Expr::Node("Multiply".to_string(), vec![
            Expr::Variable("x".to_string()),
            Expr::Node("Power".to_string(), vec![
                Expr::Variable("x".to_string()),
                Expr::Number(2)
            ])
        ]);

        let mut engine = SmocEngine::new();
        engine.add_rule("Sčítání exponentů", rule_multiply_powers);
        engine.add_rule("Sčítání čísel", smoc_arithmetics::rule_add_numbers);

        let (final_expr, steps) = engine.simplify_with_steps(&math_expr);
        
        println!("== Postup ==");
        for step in steps { println!("{}", step); }
        
        let expected = Expr::Node("Power".to_string(), vec![
            Expr::Variable("x".to_string()),
            Expr::Number(3)
        ]);

        assert_eq!(final_expr, expected);
    }
}
