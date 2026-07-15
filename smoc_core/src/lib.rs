#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Number(i64),
    Variable(String),
    Node(String, Vec<Expr>), // Univerzální stavební kámen
}

impl Expr {
    // Spočítá "velikost" stromu (tzv. LeafCount). Menší číslo = jednodušší výraz.
    pub fn complexity(&self) -> usize {
        match self {
            Expr::Number(n) => 1 + (n.abs() as f64).log10().max(0.0) as usize,
            Expr::Variable(_) => 1,
            Expr::Node(_, children) => {
                let mut sum = 1; // 1 bod za samotný uzel (např. Fraction)
                for child in children {
                    sum += child.complexity(); // Rekurzivně sečteme děti
                }
                sum
            }
        }
    }
}

// Typ pro naše matematická pravidla (Pluginy z jiných crates budou mít tento formát)
pub type RuleFn = fn(&Expr) -> Option<Expr>;
pub type PenaltyRuleFn = fn(&Expr) -> usize;

#[derive(Clone)]
pub struct NamedRule {
    pub name: String,
    pub func: RuleFn,
}

#[derive(Clone)]
pub struct NamedPenaltyRule {
    pub name: String,
    pub func: PenaltyRuleFn,
}

// Stroj, který provádí zjednodušování (The Search Engine)
#[derive(Clone)]
pub struct SmocEngine {
    rules: Vec<NamedRule>,
    penalty_rules: Vec<NamedPenaltyRule>,
}

#[derive(Clone)]
struct SearchState {
    expr: Expr,
    path: Vec<String>,
}

impl Default for SmocEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SmocEngine {
    pub fn new() -> Self {
        SmocEngine { 
            rules: Vec::new(),
            penalty_rules: Vec::new(),
        }
    }

    // Tudy se "připojují" podknihovny k naší ploše
    pub fn add_rule(&mut self, name: &str, rule: RuleFn) {
        self.rules.push(NamedRule {
            name: name.to_string(),
            func: rule,
        });
    }

    pub fn add_penalty_rule(&mut self, name: &str, rule: PenaltyRuleFn) {
        self.penalty_rules.push(NamedPenaltyRule {
            name: name.to_string(),
            func: rule,
        });
    }

    // Vyhodnocení skóre = LeafCount + Penalizace
    fn evaluate_score(&self, expr: &Expr) -> usize {
        let base_complexity = expr.complexity();
        
        fn compute_penalties(expr: &Expr, rules: &[NamedPenaltyRule]) -> usize {
            let mut penalty = 0;
            for rule in rules {
                penalty += (rule.func)(expr);
            }
            if let Expr::Node(_, children) = expr {
                for child in children {
                    penalty += compute_penalties(child, rules);
                }
            }
            penalty
        }
        
        base_complexity + compute_penalties(expr, &self.penalty_rules)
    }

    // Vrátí všechny možné nové výrazy získávané aplikací JAKÉHOKOLIV pravidla na JAKÝKOLIV poduzel 1x
    fn generate_next_states(&self, expr: &Expr) -> Vec<(Expr, String)> {
        let mut results = Vec::new();

        // 1. Zkusíme aplikovat všechna pravidla na TENTO uzel
        for rule in &self.rules {
            if let Some(new_expr) = (rule.func)(expr) {
                results.push((new_expr, rule.name.clone()));
            }
        }

        // 2. Rekurzivně zkusíme aplikovat pravidla na DĚTI
        if let Expr::Node(name, children) = expr {
            for (child_idx, child) in children.iter().enumerate() {
                let child_next_states = self.generate_next_states(child);
                for (new_child_expr, rule_name) in child_next_states {
                    let mut new_children = children.clone();
                    new_children[child_idx] = new_child_expr;
                    results.push((Expr::Node(name.clone(), new_children), rule_name));
                }
            }
        }

        results
    }

    // Nové chytré prohledávání stromu vracející i postup
    pub fn simplify_with_steps(&self, expr: &Expr) -> (Expr, Vec<String>) {
        use std::collections::{HashSet, VecDeque};

        let mut queue: VecDeque<SearchState> = VecDeque::new();
        let mut seen: HashSet<Expr> = HashSet::new();

        let initial_state = SearchState {
            expr: expr.clone(),
            path: Vec::new(),
        };

        queue.push_back(initial_state.clone());
        seen.insert(expr.clone());

        let mut best_state = initial_state.clone();
        let mut min_complexity = self.evaluate_score(&best_state.expr);
        
        let max_search_steps = 2000; // Záchranná brzda hrubé síly
        let mut iterations = 0;

        while let Some(current) = queue.pop_front() {
            iterations += 1;

            // Aktualizace dosavadního šampiona
            let current_comp = self.evaluate_score(&current.expr);
            if current_comp < min_complexity {
                min_complexity = current_comp;
                best_state = current.clone();
            }

            // Early stop: Naprostá dokonalost (číslo nebo x)
            if current_comp == 1 {
                best_state = current;
                break;
            }

            // Ochrana proti uvaření procesoru
            if iterations > max_search_steps {
                break;
            }

            // Generování dalších tahů (tvorba pavučiny osudů)
            let next_moves = self.generate_next_states(&current.expr);
            
            for (next_expr, rule_name) in next_moves {
                if !seen.contains(&next_expr) {
                    seen.insert(next_expr.clone());
                    
                    let mut new_path = current.path.clone();
                    new_path.push(rule_name);
                    
                    queue.push_back(SearchState {
                        expr: next_expr,
                        path: new_path,
                    });
                }
            }
        }

        (best_state.expr, best_state.path)
    }

    // Klasická funkce pro zpětnou kompatibilitu, která zahodí historii
    pub fn simplify(&self, expr: &Expr) -> Expr {
        self.simplify_with_steps(expr).0
    }
}



