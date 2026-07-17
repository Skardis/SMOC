use serde::{Serialize, Deserialize};
use std::sync::{OnceLock, Mutex};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Clone)]
pub struct PascalConfig {
    pub max_row_limit: usize,
    pub use_cache: bool,
}

impl Default for PascalConfig {
    fn default() -> Self {
        PascalConfig {
            max_row_limit: 10_000, // 10,000 řádků je velmi štědrý a bezpečný limit pro paměť
            use_cache: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PascalTriangle {
    pub rows: Vec<Vec<u64>>,
    pub config: PascalConfig,
}

impl PascalTriangle {
    pub fn new() -> Self {
        PascalTriangle {
            rows: vec![
                vec![1],       // n=0: (a+b)^0 = 1
                vec![1, 1],    // n=1: (a+b)^1 = 1a + 1b
            ],
            config: PascalConfig::default(),
        }
    }

    pub fn set_config(&mut self, new_config: PascalConfig) {
        self.config = new_config;
    }

    pub fn ensure_row(&mut self, n: usize) -> bool {
        let current_max = self.rows.len() - 1;
        if n <= current_max {
            return true;
        }

        if n > self.config.max_row_limit {
            return false;
        }

        if !self.config.use_cache {
            return true; // Předstíráme, že ho máme, ale spočítáme lokálně
        }

        for i in (current_max + 1)..=n {
            let prev_row = &self.rows[i - 1];
            let mut new_row = vec![1]; // Začíná vždy 1
            
            for j in 0..(prev_row.len() - 1) {
                // Předejdeme přetečení u obrovských čísel u=u64 (Pascal roste extrémně rychle!)
                let sum = prev_row[j].saturating_add(prev_row[j + 1]);
                new_row.push(sum);
            }
            
            new_row.push(1); // Končí vždy 1
            self.rows.push(new_row);
        }
        true
    }

    // Pomocná funkce pro výpočet jedné hodnoty z hlavy (bez cache)
    fn compute_binomial_local(n: usize, k: usize) -> u64 {
        if k > n { return 0; }
        if k == 0 || k == n { return 1; }
        let k = if k > n - k { n - k } else { k };
        let mut res = 1u64;
        for i in 1..=k {
            res = res.saturating_mul((n - i + 1) as u64) / (i as u64);
        }
        res
    }

    pub fn get_row(&mut self, n: usize) -> Option<Vec<u64>> {
        if n <= self.rows.len() - 1 {
            return Some(self.rows[n].clone());
        }

        if n > self.config.max_row_limit {
            return None;
        }

        if self.config.use_cache {
            if !self.ensure_row(n) { return None; }
            return Some(self.rows[n].clone());
        } else {
            // Lokální výpočet celého řádku bez ukládání do RAM
            let mut row = Vec::with_capacity(n + 1);
            for k in 0..=n {
                row.push(Self::compute_binomial_local(n, k));
            }
            return Some(row);
        }
    }
    
    pub fn binomial_coefficient(&mut self, n: usize, k: usize) -> Option<u64> {
        if k > n { return Some(0); }
        if n <= self.rows.len() - 1 {
            return Some(self.rows[n][k]);
        }
        
        if n > self.config.max_row_limit {
            return None;
        }

        if self.config.use_cache {
            if !self.ensure_row(n) { return None; }
            return Some(self.rows[n][k]);
        } else {
            Some(Self::compute_binomial_local(n, k))
        }
    }
    
    pub fn save_to_disk(&self, path: &str) -> std::io::Result<()> {
        let encoded = bincode::serialize(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let mut file = File::create(path)?;
        file.write_all(&encoded)?;
        Ok(())
    }
    
    pub fn load_from_disk(path: &str) -> Option<Self> {
        let mut file = File::open(path).ok()?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).ok()?;
        bincode::deserialize(&buffer).ok()
    }
}

// Globální Cache pro Binomické koeficienty
static PASCAL_DICT: OnceLock<Mutex<PascalTriangle>> = OnceLock::new();

pub fn get_dict() -> &'static Mutex<PascalTriangle> {
    PASCAL_DICT.get_or_init(|| Mutex::new(PascalTriangle::new()))
}

// Veřejné konfigurační rozhraní
pub fn set_pascal_config(config: PascalConfig) {
    let mut dict = get_dict().lock().unwrap();
    dict.set_config(config);
}

// Zjednodušené rozhraní pro vnější svět
pub fn get_row(n: usize) -> Option<Vec<u64>> {
    let mut dict = get_dict().lock().unwrap();
    dict.get_row(n)
}

pub fn binomial_coefficient(n: usize, k: usize) -> Option<u64> {
    let mut dict = get_dict().lock().unwrap();
    dict.binomial_coefficient(n, k)
}

pub fn load_pascal(path: &str) {
    if let Some(loaded) = PascalTriangle::load_from_disk(path) {
        let mut dict = get_dict().lock().unwrap();
        *dict = loaded;
    }
}

pub fn save_pascal(path: &str) {
    let dict = get_dict().lock().unwrap();
    let _ = dict.save_to_disk(path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pascal_triangle() {
        let mut dict = PascalTriangle::new();
        
        // (a+b)^2 = 1a^2 + 2ab + 1b^2
        let row_2 = dict.get_row(2).unwrap();
        assert_eq!(row_2, vec![1, 2, 1]);
        
        // n=5, k=2 (koeficient u a^3 b^2) => 10
        let coef = dict.binomial_coefficient(5, 2).unwrap();
        assert_eq!(coef, 10);
    }
}
