use pest_derive::Parser;
use pest::Parser;
use pest::iterators::Pair;
use smoc_core::Expr;

#[derive(Parser)]
#[grammar = "smoc.pest"]
pub struct SmocParser;

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
                            // Obyčejné překlopení znaménka rovnou převedeme na univerzální Multiply Node s -1
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
        Rule::primary => {
            let inner_pair = pair.into_inner().next().unwrap();
            build_ast(inner_pair)
        },
        Rule::number => {
            let value = pair.as_str().parse::<i64>().unwrap();
            Expr::Number(value)
        },
        Rule::letter => {
            Expr::Variable(pair.as_str().to_string())
        },
        Rule::term => {
            let mut items = Vec::new();
            for piece in pair.into_inner() {
                match piece.as_rule() {
                    Rule::letter => items.push(Expr::Variable(piece.as_str().to_string())),
                    Rule::number | Rule::fraction => items.push(build_ast(piece)),
                    _ => {}
                }
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
        _ => unreachable!(),
    }
}
