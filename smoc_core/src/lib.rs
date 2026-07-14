#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Number(i64),
    Variable(String),
    Node(String, Vec<Expr>), // Univerzální stavební kámen
}

// Typ pro naše matematická pravidla (Pluginy z jiných crates budou mít tento formát)
pub type RuleFn = fn(&Expr) -> Option<Expr>;

// Stroj, který provádí zjednodušování
pub struct SmocEngine {
    rules: Vec<RuleFn>,
}

impl SmocEngine {
    pub fn new() -> Self {
        SmocEngine { rules: Vec::new() }
    }

    // Tudy se "připojují" podknihovny k naší ploše
    pub fn add_rule(&mut self, rule: RuleFn) {
        self.rules.push(rule);
    }

    // Rekurzivně projde celý strom a aplikuje na něj všechna registrovaná pravidla
    pub fn simplify(&self, expr: &Expr) -> Expr {
        let mut current_expr = expr.clone();
        let mut changed = true;

        // Opakujeme, dokud se strom mění
        while changed {
            changed = false;
            
            // Nejdříve rekurzivně zjednodušíme děti (vnitřky uzlů)
            if let Expr::Node(name, children) = &current_expr {
                let mut new_children = Vec::new();
                for child in children {
                    new_children.push(self.simplify(child));
                }
                current_expr = Expr::Node(name.clone(), new_children);
            }

            // Teď na aktuální uzel zkusíme aplikovat všechna pravidla
            for rule in &self.rules {
                if let Some(new_expr) = rule(&current_expr) {
                    current_expr = new_expr;
                    changed = true;
                    break; // Strom se změnil, začneme aplikovat pravidla hezky od začátku
                }
            }
        }
        
        current_expr
    }
}



