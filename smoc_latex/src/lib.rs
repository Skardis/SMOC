use pest_derive::Parser;
use pest::Parser;
use pest::iterators::Pair;
use smoc_core::Expr;
use std::sync::{OnceLock, Mutex};

pub enum DecimalSeparatorMode {
    BothAreDecimals,
    DotIsDecimalCommaIgnores,
    CommaIsDecimalDotIgnores,
}

pub struct ParserConfig {
    pub decimal_mode: DecimalSeparatorMode,
}

impl Default for ParserConfig {
    fn default() -> Self {
        ParserConfig {
            decimal_mode: DecimalSeparatorMode::BothAreDecimals,
        }
    }
}

static PARSER_CONFIG: OnceLock<Mutex<ParserConfig>> = OnceLock::new();

pub fn get_parser_config() -> &'static Mutex<ParserConfig> {
    PARSER_CONFIG.get_or_init(|| Mutex::new(ParserConfig::default()))
}

pub fn set_parser_config(config: ParserConfig) {
    let mut current_config = get_parser_config().lock().unwrap();
    *current_config = config;
}

#[derive(Parser)]
#[grammar = "smoc.pest"]
pub struct SmocParser;

pub fn parse_latex(input: &str) -> Result<Expr, pest::error::Error<Rule>> {
    let mut pairs = SmocParser::parse(Rule::expression, input)?;
    Ok(build_ast(pairs.next().unwrap()))
}

pub fn build_ast(pair: Pair<Rule>) -> Expr {
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
                                Expr::Number(n) => Expr::Number(-n),
                                _ => Expr::Node("Multiply".to_string(), vec![Expr::Number(-1), expr]),
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

            if items.len() == 1 {
                items.pop().unwrap()
            } else {
                Expr::Node("Add".to_string(), items)
            }
        },
        Rule::primary | Rule::atom | Rule::group => {
            let inner_pair = pair.into_inner().next().unwrap();
            build_ast(inner_pair)
        },
        Rule::number => {
            let s = pair.as_str();
            let config = get_parser_config().lock().unwrap();
            
            // Určíme, který znak má jakou roli
            let (decimal_char, ignore_char) = match config.decimal_mode {
                DecimalSeparatorMode::BothAreDecimals => {
                    // Najdeme poslední tečku nebo čárku. Ostatní stejné znaky před tím budou divné, ale my prostě vezmeme poslední.
                    // Ale lépe: Prostě jakoukoliv tečku/čárku bereme jako desetinnou.
                    // Vzhledem k tomu, že PEST může sežrat víc čárek, použijeme první výskyt a zbytek ořízneme nebo necháme padnout parse()
                    // Pro zjednodušení převedeme vše na tečku.
                    ('.', ' ') // ' ' je dummy, protože nic neignorujeme
                },
                DecimalSeparatorMode::DotIsDecimalCommaIgnores => ('.', ','),
                DecimalSeparatorMode::CommaIsDecimalDotIgnores => (',', '.'),
            };

            let mut processed = s.to_string();

            // Pokud ignorujeme nějaký znak (např. tisícové oddělovače), tak ho smažeme
            if ignore_char != ' ' {
                processed = processed.replace(ignore_char, "");
            }

            // Pokud BothAreDecimals, tak čárku převedeme na tečku
            if matches!(config.decimal_mode, DecimalSeparatorMode::BothAreDecimals) {
                processed = processed.replace(',', ".");
            }

            if processed.contains(decimal_char) {
                // Je to desetinné číslo!
                let parts: Vec<&str> = processed.split(decimal_char).collect();
                if parts.len() == 2 {
                    let int_part = parts[0];
                    let frac_part = parts[1];
                    let full_num_str = format!("{}{}", int_part, frac_part);
                    
                    if let Ok(top) = full_num_str.parse::<i64>() {
                        let bottom: i64 = 10_i64.pow(frac_part.len() as u32);
                        return Expr::Node("Fraction".to_string(), vec![Expr::Number(top), Expr::Number(bottom)]);
                    }
                }
            }

            // Pokud to není desetinné, nebo se to nepovedlo parsovat (např. vícero teček)
            let clean_str = processed.replace(decimal_char, "");
            let value = clean_str.parse::<i64>().unwrap_or(0);
            Expr::Number(value)
        },
        Rule::letter => {
            Expr::Variable(pair.as_str().to_string())
        },
        Rule::term => {
            let mut items = Vec::new();
            for piece in pair.into_inner() {
                if piece.as_str() == "\\cdot" || piece.as_str() == "*" {
                    continue;
                }
                items.push(build_ast(piece));
            }
            if items.len() == 1 {
                items.pop().unwrap()
            } else {
                Expr::Node("Multiply".to_string(), items)
            }
        },
        Rule::fraction => {
            let mut inner = pair.into_inner();
            let top_expr = build_ast(inner.next().unwrap());
            let bottom_expr = build_ast(inner.next().unwrap());
            Expr::Node("Fraction".to_string(), vec![top_expr, bottom_expr])
        },
        Rule::power => {
            let mut atoms = Vec::new();
            for piece in pair.into_inner() {
                atoms.push(build_ast(piece));
            }
            
            // X^Y^Z se v matematice počítá zprava doleva jako X^(Y^Z)
            let mut expr = atoms.pop().unwrap();
            while let Some(left) = atoms.pop() {
                expr = Expr::Node("Power".to_string(), vec![left, expr]);
            }
            expr
        },
        Rule::sqrt => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();
            
            let degree;
            let content;
            
            if first.as_rule() == Rule::sqrt_degree {
                degree = build_ast(first.into_inner().next().unwrap());
                content = build_ast(inner.next().unwrap());
            } else {
                degree = Expr::Number(2);
                content = build_ast(first);
            }
            
            Expr::Node("Power".to_string(), vec![
                content,
                Expr::Node("Fraction".to_string(), vec![Expr::Number(1), degree])
            ])
        },
        _ => unreachable!(),
    }
}
