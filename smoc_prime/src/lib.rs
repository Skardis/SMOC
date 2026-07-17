use serde::{Serialize, Deserialize};
use std::sync::{OnceLock, Mutex};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Clone)]
pub struct PrimeConfig {
    pub max_limit: u64,
    pub use_cache: bool,
}

impl Default for PrimeConfig {
    fn default() -> Self {
        PrimeConfig {
            max_limit: 10_000_000, // Výchozí bezpečný limit (10 milionů)
            use_cache: true,       // Výchozí je ukládat do RAM
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PrimeDictionary {
    pub primes: Vec<u64>,
    pub max_checked: u64,
    pub config: PrimeConfig,
}

impl PrimeDictionary {
    pub fn new() -> Self {
        PrimeDictionary {
            primes: vec![2, 3],
            max_checked: 3,
            config: PrimeConfig::default(),
        }
    }

    pub fn set_config(&mut self, new_config: PrimeConfig) {
        self.config = new_config;
    }

    pub fn extend_limit(&mut self, limit: u64) -> bool {
        if limit <= self.max_checked {
            return true;
        }
        
        // Zabráníme OOM / zhroucení pro obří čísla!
        if limit > self.config.max_limit {
            return false;
        }

        // Pokud je cachování vypnuto, nebudeme Sítko zvětšovat v paměti
        if !self.config.use_cache {
            return true; // Předstíráme úspěch, protože výpočet se provede lokálně "on-the-fly"
        }

        let mut is_prime = vec![true; (limit + 1) as usize];
        is_prime[0] = false;
        is_prime[1] = false;

        let max_root = (limit as f64).sqrt() as usize;
        for p in 2..=max_root {
            if is_prime[p] {
                let mut multiple = p * p;
                while multiple <= limit as usize {
                    is_prime[multiple] = false;
                    multiple += p;
                }
            }
        }

        self.primes.clear();
        for (i, &prime) in is_prime.iter().enumerate() {
            if prime {
                self.primes.push(i as u64);
            }
        }
        self.max_checked = limit;
        true
    }

    pub fn is_prime(&mut self, n: u64) -> Option<bool> {
        if n <= self.max_checked {
            return Some(self.primes.binary_search(&n).is_ok());
        }

        if n > self.config.max_limit {
            return None; // Mimo nastavený limit, odmítnuto!
        }

        if self.config.use_cache {
            if !self.extend_limit(n) { return None; }
            return Some(self.primes.binary_search(&n).is_ok());
        } else {
            // Cachování je vypnuté - provedeme pomalý lokální výpočet Trial Division
            let max_root = (n as f64).sqrt() as u64;
            for p in 2..=max_root {
                if n % p == 0 { return Some(false); }
            }
            return Some(true);
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

static PRIME_DICT: OnceLock<Mutex<PrimeDictionary>> = OnceLock::new();

pub fn get_dict() -> &'static Mutex<PrimeDictionary> {
    PRIME_DICT.get_or_init(|| Mutex::new(PrimeDictionary::new()))
}

// Veřejné konfigurační rozhraní
pub fn set_prime_config(config: PrimeConfig) {
    let mut dict = get_dict().lock().unwrap();
    dict.set_config(config);
}

pub fn prime_factors(mut n: i64) -> Option<Vec<i64>> {
    if n <= 1 {
        return Some(vec![n]);
    }
    
    let limit = (n as f64).sqrt() as u64 + 1;
    let mut dict = get_dict().lock().unwrap();
    
    if limit > dict.config.max_limit {
        return None; // Ochrana: Číslo je tak velké, že i odmocnina přesahuje náš limit!
    }
    
    let mut factors = Vec::new();

    if dict.config.use_cache {
        dict.extend_limit(limit);
        for &p in &dict.primes {
            if p as i64 > n { break; }
            while n % (p as i64) == 0 {
                factors.push(p as i64);
                n /= p as i64;
            }
        }
    } else {
        // Lokální výpočet bez uložení do paměti (pomalejší, ale bez paměti)
        let mut p = 2;
        while p * p <= n {
            while n % p == 0 {
                factors.push(p);
                n /= p;
            }
            p += 1;
        }
    }

    if n > 1 {
        factors.push(n);
    }
    Some(factors)
}

pub fn load_primes(path: &str) {
    if let Some(loaded) = PrimeDictionary::load_from_disk(path) {
        let mut dict = get_dict().lock().unwrap();
        *dict = loaded;
    }
}

pub fn save_primes(path: &str) {
    let dict = get_dict().lock().unwrap();
    let _ = dict.save_to_disk(path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prime_factors() {
        let factors = prime_factors(15).unwrap();
        assert_eq!(factors, vec![3, 5]);

        let factors2 = prime_factors(1024).unwrap();
        assert_eq!(factors2, vec![2; 10]);

        let factors3 = prime_factors(97).unwrap(); // 97 je prvočíslo
        assert_eq!(factors3, vec![97]);
    }

    #[test]
    fn test_is_prime() {
        let mut dict = PrimeDictionary::new();
        assert_eq!(dict.is_prime(97), Some(true));
        assert_eq!(dict.is_prime(100), Some(false));
    }
}
