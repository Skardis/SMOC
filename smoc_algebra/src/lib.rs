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

// Pravidlo: (a * b * c)^n = a^n * b^n * c^n
pub fn rule_power_over_multiplication(expr: &Expr) -> Option<Expr> {
    if let Expr::Node(name, items) = expr {
        if name == "Power" && items.len() == 2 {
            let base = &items[0];
            let exp = &items[1];
            
            if let Expr::Node(base_name, base_items) = base {
                if base_name == "Multiply" {
                    let mut new_items = Vec::new();
                    for item in base_items {
                        new_items.push(Expr::Node("Power".to_string(), vec![item.clone(), exp.clone()]));
                    }
                    return Some(Expr::Node("Multiply".to_string(), new_items));
                }
            }
        }
    }
    None
}

// Pravidlo: x^a / x^b = x^{a-b}
pub fn rule_divide_powers(expr: &Expr) -> Option<Expr> {
    if let Expr::Node(name, items) = expr {
        if name == "Fraction" && items.len() == 2 {
            let top = &items[0];
            let bottom = &items[1];

            fn extract_bases(e: &Expr) -> std::collections::HashMap<Expr, Expr> {
                let mut map = std::collections::HashMap::new();
                match e {
                    Expr::Node(n, children) if n == "Multiply" => {
                        for child in children {
                            if let Expr::Node(p, p_children) = child {
                                if p == "Power" && p_children.len() == 2 {
                                    map.insert(p_children[0].clone(), p_children[1].clone());
                                } else {
                                    map.insert(child.clone(), Expr::Number(1));
                                }
                            } else {
                                map.insert(child.clone(), Expr::Number(1));
                            }
                        }
                    },
                    Expr::Node(p, p_children) if p == "Power" && p_children.len() == 2 => {
                        map.insert(p_children[0].clone(), p_children[1].clone());
                    },
                    other => {
                        map.insert(other.clone(), Expr::Number(1));
                    }
                }
                map
            }

            let mut top_bases = extract_bases(top);
            let mut bottom_bases = extract_bases(bottom);
            let mut changed = false;

            let bases: Vec<Expr> = top_bases.keys().cloned().collect();
            for base in bases {
                if let Some(bottom_exp) = bottom_bases.remove(&base) {
                    if let Some(top_exp) = top_bases.remove(&base) {
                        changed = true;
                        let new_exp = Expr::Node("Add".to_string(), vec![
                            top_exp,
                            Expr::Node("Multiply".to_string(), vec![Expr::Number(-1), bottom_exp])
                        ]);
                        top_bases.insert(base, new_exp);
                    }
                }
            }

            if changed {
                let mut new_top_items = Vec::new();
                for (b, e) in top_bases {
                    if e == Expr::Number(1) {
                        new_top_items.push(b);
                    } else if e == Expr::Number(0) {
                        // zmizí
                    } else {
                        new_top_items.push(Expr::Node("Power".to_string(), vec![b, e]));
                    }
                }
                
                let mut new_bottom_items = Vec::new();
                for (b, e) in bottom_bases {
                    if e == Expr::Number(1) {
                        new_bottom_items.push(b);
                    } else if e == Expr::Number(0) {
                        // zmizí
                    } else {
                        new_bottom_items.push(Expr::Node("Power".to_string(), vec![b, e]));
                    }
                }

                // Chceme zajistit, aby čísla byla vždy na začátku pole u Multiply
                new_top_items.sort_by_key(|a| if let Expr::Number(_) = a { 0 } else { 1 });
                new_bottom_items.sort_by_key(|a| if let Expr::Number(_) = a { 0 } else { 1 });

                let new_top = if new_top_items.is_empty() { Expr::Number(1) } else if new_top_items.len() == 1 { new_top_items.pop().unwrap() } else { Expr::Node("Multiply".to_string(), new_top_items) };
                let new_bottom = if new_bottom_items.is_empty() { Expr::Number(1) } else if new_bottom_items.len() == 1 { new_bottom_items.pop().unwrap() } else { Expr::Node("Multiply".to_string(), new_bottom_items) };

                if new_bottom == Expr::Number(1) {
                    return Some(new_top);
                } else {
                    return Some(Expr::Node("Fraction".to_string(), vec![new_top, new_bottom]));
                }
            }
        }
    }
    None
}

// Pravidlo: Binomická věta (a+b)^n = sum(binom(n, k) * a^{n-k} * b^k)
pub fn rule_binomial_expansion(expr: &Expr) -> Option<Expr> {
    if let Expr::Node(name, items) = expr {
        if name == "Power" && items.len() == 2 {
            let base = &items[0];
            let exp = &items[1];

            if let Expr::Node(base_name, base_items) = base {
                if base_name == "Add" && base_items.len() == 2 {
                    let a = &base_items[0];
                    let b = &base_items[1];

                    match exp {
                        Expr::Number(n) => {
                            if *n > 0 && *n <= 15 {
                                let n_usize = *n as usize;
                                let row = smoc_pascaltriangle::get_row(n_usize)?;
                                let mut expanded_terms = Vec::new();

                                for k in 0..=n_usize {
                                    let coef = row[k];
                                    let exp_a = (n_usize - k) as i64;
                                    let exp_b = k as i64;

                                    let mut term_parts = Vec::new();
                                    if coef != 1 {
                                        term_parts.push(Expr::Number(coef as i64));
                                    }
                                    if exp_a != 0 {
                                        if exp_a == 1 {
                                            term_parts.push(a.clone());
                                        } else {
                                            term_parts.push(Expr::Node("Power".to_string(), vec![a.clone(), Expr::Number(exp_a)]));
                                        }
                                    }
                                    if exp_b != 0 {
                                        if exp_b == 1 {
                                            term_parts.push(b.clone());
                                        } else {
                                            term_parts.push(Expr::Node("Power".to_string(), vec![b.clone(), Expr::Number(exp_b)]));
                                        }
                                    }

                                    if term_parts.is_empty() {
                                        expanded_terms.push(Expr::Number(1));
                                    } else if term_parts.len() == 1 {
                                        expanded_terms.push(term_parts.pop().unwrap());
                                    } else {
                                        expanded_terms.push(Expr::Node("Multiply".to_string(), term_parts));
                                    }
                                }

                                return Some(Expr::Node("Add".to_string(), expanded_terms));
                            } else if *n < 0 && *n >= -15 {
                                // Záporný exponent: (a+b)^{-n} = 1 / (a+b)^n
                                let pos_power = Expr::Node("Power".to_string(), vec![base.clone(), Expr::Number(-*n)]);
                                return Some(Expr::Node("Fraction".to_string(), vec![Expr::Number(1), pos_power]));
                            }
                        },
                        Expr::Node(exp_name, exp_items) if exp_name == "Fraction" && exp_items.len() == 2 => {
                            // Zlomkový exponent
                            if let (Expr::Number(top), Expr::Number(bottom)) = (&exp_items[0], &exp_items[1]) {
                                if *top > 1 && *top <= 15 {
                                    // Rozložíme čitatel pod odmocninou: (a+b)^{top/bottom} = ((a+b)^top)^{1/bottom}
                                    let inner_power = Expr::Node("Power".to_string(), vec![base.clone(), Expr::Number(*top)]);
                                    let root_exp = Expr::Node("Fraction".to_string(), vec![Expr::Number(1), Expr::Number(*bottom)]);
                                    return Some(Expr::Node("Power".to_string(), vec![inner_power, root_exp]));
                                } else if *top < 0 && *top >= -15 {
                                    // Záporný zlomkový exponent: (a+b)^{-top/bottom} = 1 / ((a+b)^top)^{1/bottom}
                                    let inner_power = Expr::Node("Power".to_string(), vec![base.clone(), Expr::Number(-*top)]);
                                    let root_exp = Expr::Node("Fraction".to_string(), vec![Expr::Number(1), Expr::Number(*bottom)]);
                                    let full_power = Expr::Node("Power".to_string(), vec![inner_power, root_exp]);
                                    return Some(Expr::Node("Fraction".to_string(), vec![Expr::Number(1), full_power]));
                                }
                            }
                        },
                        _ => {}
                    }
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
