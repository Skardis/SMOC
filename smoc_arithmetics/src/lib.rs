use smoc_core::Expr;

pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a.abs()
}

// Pravidlo 1: Vytýkání ze sčítání
pub fn rule_factorize_add(expr: &Expr) -> Option<Expr> {
    if let Expr::Node(name, items) = expr {
        if name == "Add" {
            let mut common_gcd = 0;
            
            for item in items {
                let item_num = match item {
                    Expr::Number(n) => n.abs(),
                    Expr::Node(sub_name, sub_items) if sub_name == "Multiply" => {
                        let mut total_coeff = 1;
                        for piece in sub_items {
                            if let Expr::Number(val) = piece {
                                total_coeff *= val;
                            }
                        }
                        total_coeff.abs()
                    },
                    _ => 1,
                };

                if common_gcd == 0 {
                    common_gcd = item_num;
                } else {
                    common_gcd = gcd(common_gcd, item_num);
                }
            }

            if common_gcd > 1 {
                let mut new_items = Vec::new();
                for item in items {
                    let new_item = match item {
                        Expr::Number(n) => Expr::Number(n / common_gcd),
                        Expr::Node(sub_name, sub_items) if sub_name == "Multiply" => {
                            let mut total_coeff = 1;
                            let mut new_mult = Vec::new();
                            for piece in sub_items {
                                if let Expr::Number(val) = piece {
                                    total_coeff *= val;
                                } else {
                                    new_mult.push(piece.clone());
                                }
                            }
                            let final_coeff = total_coeff / common_gcd;
                            new_mult.insert(0, Expr::Number(final_coeff));
                            Expr::Node("Multiply".to_string(), new_mult)
                        },
                        other => other.clone(),
                    };
                    new_items.push(new_item);
                }
                
                return Some(Expr::Node("Multiply".to_string(), vec![
                    Expr::Number(common_gcd),
                    Expr::Node("Add".to_string(), new_items)
                ]));
            }
        }
    }
    None
}

// Pravidlo 2: Krácení zlomků
pub fn rule_simplify_fraction(expr: &Expr) -> Option<Expr> {
    if let Expr::Node(name, items) = expr {
        if name == "Fraction" && items.len() == 2 {
            let top = &items[0];
            let bottom = &items[1];

            fn extract_parts(expr: &Expr) -> (i64, Vec<Expr>) {
                match expr {
                    Expr::Node(name, items) if name == "Multiply" => {
                        let mut coeff = 1;
                        let mut rest = Vec::new();
                        for item in items {
                            if let Expr::Number(n) = item {
                                coeff *= n;
                            } else {
                                rest.push(item.clone());
                            }
                        }
                        (coeff, rest)
                    },
                    Expr::Number(n) => (*n, vec![]),
                    other => (1, vec![other.clone()])
                }
            }

            let (mut top_coeff, mut top_rest) = extract_parts(top);
            let (mut bottom_coeff, mut bottom_rest) = extract_parts(bottom);

            let common = gcd(top_coeff.abs(), bottom_coeff.abs());
            
            let mut will_change = false;
            if common > 1 { will_change = true; }
            if bottom_coeff < 0 { will_change = true; }

            let mut i = 0;
            while i < top_rest.len() {
                if let Some(pos) = bottom_rest.iter().position(|x| x == &top_rest[i]) {
                    top_rest.remove(i);
                    bottom_rest.remove(pos);
                    will_change = true;
                } else {
                    i += 1;
                }
            }

            if !will_change {
                return None;
            }

            top_coeff /= common;
            bottom_coeff /= common;
            
            if bottom_coeff < 0 {
                top_coeff = -top_coeff;
                bottom_coeff = -bottom_coeff;
            }

            let mut final_top = vec![Expr::Number(top_coeff)];
            final_top.extend(top_rest);
            let top_expr = if final_top.len() == 1 { final_top.pop().unwrap() } else { Expr::Node("Multiply".to_string(), final_top) };

            if bottom_coeff == 1 && bottom_rest.is_empty() {
                return Some(top_expr);
            } else {
                let mut final_bottom = vec![Expr::Number(bottom_coeff)];
                final_bottom.extend(bottom_rest);
                let bottom_expr = if final_bottom.len() == 1 { final_bottom.pop().unwrap() } else { Expr::Node("Multiply".to_string(), final_bottom) };
                
                return Some(Expr::Node("Fraction".to_string(), vec![top_expr, bottom_expr]));
            }
        }
    }
    None
}

// Pravidlo 3: Obyčejné sčítání čísel (např. 1+2 = 3)
pub fn rule_add_numbers(expr: &Expr) -> Option<Expr> {
    if let Expr::Node(name, items) = expr {
        if name == "Add" {
            let mut all_numbers = true;
            let mut sum = 0;
            for item in items {
                if let Expr::Number(n) = item {
                    sum += n;
                } else {
                    all_numbers = false;
                    break;
                }
            }
            if all_numbers && items.len() > 1 {
                return Some(Expr::Number(sum));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use smoc_core::SmocEngine;

    #[test]
    fn test_engine_rules() {
        let math_expr = Expr::Node("Fraction".to_string(), vec![
            Expr::Node("Add".to_string(), vec![
                Expr::Node("Multiply".to_string(), vec![Expr::Number(4), Expr::Variable("x".to_string())]),
                Expr::Node("Multiply".to_string(), vec![Expr::Number(8), Expr::Variable("y".to_string())]),
            ]),
            Expr::Node("Add".to_string(), vec![
                Expr::Node("Multiply".to_string(), vec![Expr::Number(2), Expr::Variable("x".to_string())]),
                Expr::Node("Multiply".to_string(), vec![Expr::Number(4), Expr::Variable("y".to_string())]),
            ]),
        ]);

        let mut engine = SmocEngine::new();
        engine.add_rule("Vytýkání ze sčítání", rule_factorize_add);
        engine.add_rule("Krácení zlomků", rule_simplify_fraction);

        let (final_expr, steps) = engine.simplify_with_steps(&math_expr);
        
        println!("== Nalezený postup ==");
        for (i, step) in steps.iter().enumerate() {
            println!("Krok {}: {}", i + 1, step);
        }
        
        assert_eq!(final_expr, Expr::Number(2));
    }
}
