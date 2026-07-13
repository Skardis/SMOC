use pest_derive::Parser;
use pest::Parser;
// Z pest knihovny si vytáhneme typ Pair
use pest::iterators::Pair;

#[derive(Parser)]
#[grammar = "smoc.pest"] // Tady mu řekneme, kde leží náš soubor s pravidly
pub struct SmocParser;

#[derive(Debug, Clone, PartialEq, Eq)]

pub enum Expr {
    // 1. Výraz může být obyčejné celé číslo
    Integer(i64),
    
    // 2. Může to být proměnná (jen čisté písmenko, bez koeficientu!)
    Variable(String),
    
    // 3. Zlomek
    Fraction {
        numerator: Box<Expr>,
        denominator: Box<Expr>,
    },
    
    // 4. Násobení (pole Výrazů, které se mezi sebou násobí)
    Multiply(Vec<Expr>),
    
    // 5. Sčítání (pole Výrazů, které se mezi sebou sčítají)
    Add(Vec<Expr>),
}
impl Expr {
    // Pomocná funkce (konstruktor) pro snadné vytvoření zlomku
    pub fn new_fraction(numerator: Expr, denominator: Expr) -> Self {
        // Zde vracíme naši variantu Fraction z enumu Expr
        // Pomocí Box::new() si ty výrazy zabalíme až tady uvnitř
        Expr::Fraction {
            numerator: Box::new(numerator),
            denominator: Box::new(denominator),
        }
    }
    
    // Pro pohodlí si přidáme i konstruktor pro číslo
    pub fn new_integer(value: i64) -> Self {
        Expr::Integer(value)
    }

    pub fn new_variable(name: &str) -> Self {
        Expr::Variable(name.to_string())
    }

    // Nový konstruktor pro násobení
    pub fn new_multiply(exprs: Vec<Expr>) -> Self {
        Expr::Multiply(exprs)
    }

    // Nový konstruktor pro sčítání
    pub fn new_add(exprs: Vec<Expr>) -> Self {
        Expr::Add(exprs)
    }

    pub fn simplify(self) -> Self {
        // Co vlastně aktuálně jsem?
        match self {
            
            // 1. JSEM ZLOMEK! (Rozbalíme si jeho vnitřnosti)
            Expr::Fraction { numerator, denominator } => {
                
                // Nejdřív rekurzivně zjednodušíme čitatel i jmenovatel
                let num_simp = numerator.simplify();
                let den_simp = denominator.simplify();

                Self::simplify_fraction(num_simp.factorize(), den_simp.factorize())
            },
            
            // Pro Násobení zavoláme simplify rekurzivně na všechny prvky v poli
            Expr::Multiply(items) => {
                let simplified_items: Vec<Expr> = items.into_iter().map(|item| item.simplify()).collect();
                Expr::new_multiply(simplified_items)
            },
            
            // Pro Sčítání zavoláme simplify rekurzivně na všechny prvky v poli
            Expr::Add(items) => {
                let simplified_items: Vec<Expr> = items.into_iter().map(|item| item.simplify()).collect();
                Expr::new_add(simplified_items)
            },
            
            // Číslo a Proměnná se už zjednodušit nedají
            other => other,
        }
    }    
    pub fn factorize(self) -> Self {
        match self {
            // Zajímá nás jen vytýkání ze sčítání
            Expr::Add(items) => {
                // KROK 1: Zjistit, jakého největšího společného dělitele
                // mají všechny prvky uvnitř tohoto sčítání.
                let mut common_gcd = 0;
                
                for item in &items {
                    // CÍL PRO TEBE: Vytáhnout z "item" číslo a uložit ho do proměnné "item_num".
                    // "item" může být buď Expr::Integer, nebo Expr::Multiply, 
                    // ve kterém je Expr::Integer schovaný jako první prvek.
                    // Pokud tam číslo není, item_num by mělo být 1.
                    
                    let item_num = match item {
                        // Je to čisté číslo? Vezmeme ho (v absolutní hodnotě)
                        Expr::Integer(n) => n.abs(),
                        
                        // Je to Násobení?
                        Expr::Multiply(n) => {
                            let mut total_coeff = 1;
                            for piece in n {
                                if let Expr::Integer(val) = piece {
                                    total_coeff *= val;
                                }
                            }
                            total_coeff.abs()
                        },

                        
                        // Všechno ostatní (např. Expr::Variable) bere koeficient 1
                        _ => 1,
                    };

                    
                    // Aktualizujeme náš společný dělitel
                    if common_gcd == 0 {
                        common_gcd = item_num;
                    } else {
                        common_gcd = gcd(common_gcd, item_num);
                    }
                }
                
                // KROK 2: Máme dělitele většího než 1?
                if common_gcd > 1 {
                    let mut new_items = Vec::new();
                    
                    for item in items {
                        let new_item = match item {
                            // Čisté číslo rovnou vydělíme
                            Expr::Integer(n) => Expr::new_integer(n / common_gcd),
                            
                            // U násobení sáhneme na první prvek. 
                            // Použijeme first_mut(), abychom mohli prvek uvnitř rovnou změnit!
                            Expr::Multiply(mult_list) => {
                                let mut total_coeff = 1;
                                let mut new_mult = Vec::new();
                                
                                // Posbíráme všechna čísla do jednoho velkého a písmenka si necháme
                                for piece in mult_list {
                                    if let Expr::Integer(val) = piece {
                                        total_coeff *= val;
                                    } else {
                                        new_mult.push(piece); // Tohle je písmenko/zlomek
                                    }
                                }
                                
                                // Tady se stane to hlavní kouzlo!
                                let final_coeff = total_coeff / common_gcd;
                                
                                // Vložíme naše nové vykrácené číslo úplně na začátek
                                new_mult.insert(0, Expr::new_integer(final_coeff));
                                
                                Expr::Multiply(new_mult)
                            },


                            
                            other => other,
                        };
                        new_items.push(new_item);
                    }
                    
                    // Všechno máme vydělené v new_items.
                    // Teď to slepíme! Výsledek je: společný_dělitel * (new_items)
                    Expr::new_multiply(vec![
                        Expr::new_integer(common_gcd),
                        Expr::new_add(new_items)
                    ])
                } else {
                    // Žádný společný dělitel nebyl nalezen, vrátíme to tak, jak to přišlo
                    Expr::Add(items)
                }
            },
            
            // Pokud to není Add, nemáme co vytýkat, vrátíme beze změny
            other => other,
        }
    }
        // FÁZE 2: Totální destrukce zlomku
    pub fn simplify_fraction(top: Expr, bottom: Expr) -> Expr {
        // Pomocná vnitřní minulačka: rozebere výraz na číslo a zbytek
        fn extract_parts(expr: Expr) -> (i64, Vec<Expr>) {
            match expr {
                Expr::Multiply(items) => {
                    let mut coeff = 1;
                    let mut rest = Vec::new();
                    for item in items {
                        if let Expr::Integer(n) = item {
                            coeff *= n;
                        } else {
                            rest.push(item);
                        }
                    }
                    (coeff, rest)
                },
                Expr::Integer(n) => (n, vec![]), // Je to jen číslo
                other => (1, vec![other]) // Není tam číslo, takže koeficient je 1
            }
        }

        // 1. Rozložení obou stran
        let (mut top_coeff, mut top_rest) = extract_parts(top);
        let (mut bottom_coeff, mut bottom_rest) = extract_parts(bottom);

        // 2. Krácení čísel pomocí starého dobrého GCD
        let common = gcd(top_coeff.abs(), bottom_coeff.abs());
        top_coeff /= common;
        bottom_coeff /= common;
        
        // Estetika: Pokud je dole mínus, hodíme ho radši nahoru (např. 1/-2 -> -1/2)
        if bottom_coeff < 0 {
            top_coeff = -top_coeff;
            bottom_coeff = -bottom_coeff;
        }

        // 3. Škrtání stejných věcí (to nejlepší z matematiky!)
        let mut i = 0;
        while i < top_rest.len() {
            // Hledáme, jestli se prvek z vršku nachází i dole
            if let Some(pos) = bottom_rest.iter().position(|x| x == &top_rest[i]) {
                // SHODA! Nemilosrdně to z obou stran vymažeme
                top_rest.remove(i);
                bottom_rest.remove(pos);
            } else {
                i += 1; // Jdeme na další prvek
            }
        }

        // 4. Slepení zpět dohromady
        let mut final_top = vec![Expr::new_integer(top_coeff)];
        final_top.extend(top_rest); // Přidáme zbytek
        
        // Pokud zbylo jen číslo, dáme ho samotné. Jinak použijeme Multiply.
        let top_expr = if final_top.len() == 1 { final_top.pop().unwrap() } else { Expr::Multiply(final_top) };

        if bottom_coeff == 1 && bottom_rest.is_empty() {
            // Spodek je 1, zlomek zaniká!
            top_expr
        } else {
            // Spodek ještě existuje, vytvoříme Multiply a nový zlomek
            let mut final_bottom = vec![Expr::new_integer(bottom_coeff)];
            final_bottom.extend(bottom_rest);
            let bottom_expr = if final_bottom.len() == 1 { final_bottom.pop().unwrap() } else { Expr::Multiply(final_bottom) };
            
            Expr::new_fraction(top_expr, bottom_expr)
        }
    }

}
pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    // Dokud nemáme nulu ve jmenovateli
    while b != 0 {
        // Schováme si béčko
        let temp = b;
        // Do béčka dáme zbytek po dělení.
        // Příklad: Pokud a=28 a b=12, tak zbytek po dělení je 4. 
        // V dalším kroku se budeme ptát na a=12 a b=4. A zbytek je 0. Hotovo!
        b = a % b;
        // Do áčka dáme schované staré béčko
        a = temp;
    }
    // Na konci nám výsledek zůstane v "a". Vrátíme ho vždy kladný.
    a.abs()
}

// Funkce vezme jeden "kousek" z parseru a udělá z něj náš Výraz
pub fn build_ast(pair: Pair<Rule>) -> Expr {
    // Podle jakého pravidla to parser poznal?
    match pair.as_rule() {
        Rule::expression => {
            let mut items = Vec::new();
            let mut is_negative = false;

            for piece in pair.into_inner() {
                match piece.as_rule() {
                    Rule::primary => {
                        let mut expr = build_ast(piece);
                        
                        if is_negative {
                            expr = match expr {
                                Expr::Integer(n) => Expr::new_integer(-n),
                                Expr::Multiply(mut mult_items) => {
                                    let mut found_number = false;
                                    for item in mult_items.iter_mut() {
                                        if let Expr::Integer(n) = item {
                                            *n = -(*n);
                                            found_number = true;
                                            break;
                                        }
                                    }
                                    if !found_number {
                                        mult_items.insert(0, Expr::new_integer(-1));
                                    }
                                    Expr::Multiply(mult_items)
                                },
                                other => Expr::new_multiply(vec![Expr::new_integer(-1), other]),
                            };
                            is_negative = false;
                        }
                        items.push(expr);
                    },
                    Rule::operation => {
                        if piece.as_str() == "-" {
                            is_negative = true;
                        } else {
                            is_negative = false;
                        }
                    },
                    _ => {}
                }
            }

            // Pokud je tam jen jeden výraz, vrátíme ho rovnou. Jinak to zabalíme do sčítání.
            if items.len() == 1 {
                items.pop().unwrap()
            } else {
                Expr::new_add(items)
            }
        },
        Rule::primary => {
            // Obal primary pošleme dál
            let inner_pair = pair.into_inner().next().unwrap();
            build_ast(inner_pair)
        },
        Rule::number => {
            let value = pair.as_str().parse::<i64>().unwrap();
            Expr::new_integer(value)
        },
        Rule::letter => {
            // Samotné písmenko je teď naše proměnná
            Expr::new_variable(pair.as_str())
        },
        Rule::term => {
            // Term (např. 3xy) se skládá z více věcí
            let mut items = Vec::new();
            
            for piece in pair.into_inner() {
                match piece.as_rule() {
                    Rule::letter => items.push(Expr::new_variable(piece.as_str())),
                    Rule::number | Rule::fraction => items.push(build_ast(piece)),
                    _ => {}
                }
            }
            
            // Pokud to bylo jenom např. "x", vrátíme rovnou x. 
            // Jinak to zabalíme do Násobení!
            if items.len() == 1 {
                items.pop().unwrap()
            } else {
                Expr::new_multiply(items)
            }
        },
        Rule::fraction => {
            let mut inner = pair.into_inner();
            let top_expr = build_ast(inner.next().unwrap());
            let bottom_expr = build_ast(inner.next().unwrap());
            Expr::new_fraction(top_expr, bottom_expr)
        },
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    // Načteme si vše z našeho souboru
    use super::*;

    #[test]
    fn test_create_expression() {
        // Vytvoříme si čitatele: 28
        let top = Expr::new_integer(28);
        
        // Vytvoříme si jmenovatele: proměnná "x"
        let bottom = Expr::new_variable("x");
        
        // Zabalíme to do zlomku: 28 / x
        let my_fraction = Expr::new_fraction(top, bottom);
        
        // Vypíšeme si, jak náš stromeček (AST) vypadá uvnitř
        println!("{:#?}", my_fraction);
    }    
    #[test]
    fn test_gcd() {
        assert_eq!(gcd(28, 12), 4);
        assert_eq!(gcd(100, 10), 10);
        assert_eq!(gcd(7, 3), 1); // Prvočísla nemají společného dělitele (kromě 1)
    }
    #[test]
    fn test_simplify() {
        // Vytvoříme si čitatele: 28
        let top = Expr::new_integer(28);
        
        // Vytvoříme si jmenovatele: proměnná "x"
        let bottom = Expr::new_integer(12);
        
        // Zabalíme to do zlomku: 28 / x
        let my_fraction = Expr::new_fraction(top, bottom);
        
        // Vypíšeme si, jak náš stromeček (AST) vypadá uvnitř
        println!("{:#?}", my_fraction.simplify());
    }    
    #[test]
    fn test_parse_text() {
        // Vytvoříme ten zlomek, o kterém jsme mluvili!
        // Vršek: 4x + 8y
        // Spodek: 2x + 4y
        let input = r#"\frac{4x + 8y}{2x + 4y}"#;
        
        // Zkusíme ho rozeznat! Náš parser to přečte jako celý jeden zlomek.
        let parse_result = SmocParser::parse(Rule::expression, input);
        assert!(parse_result.is_ok(), "Jejda, parser to nepřečetl!");
        
        let first_pair = parse_result.unwrap().next().unwrap();
        let math_expr = build_ast(first_pair);
        
        println!("Takto vypadá načtený OBR Zlomek:");
        println!("{:#?}", math_expr);
        
        println!("A takto po drastickém ZJEDNODUŠENÍ (Mělo by zbýt jen číslo 2):");
        println!("{:#?}", math_expr.simplify());
    }



}


