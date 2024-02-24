use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct Blocked {
    pub blocked: bool,
    pub reset: u128,
}

#[derive(Debug)]
pub struct EphemeralCache {
    pub cache: HashMap<String, u128>,
}

impl EphemeralCache {
    pub fn new() -> EphemeralCache {
        EphemeralCache {
            cache: HashMap::new(),
        }
    }

    pub fn is_blocked(&self, identifier: &str) -> Blocked {
        let reset = self.cache.get(identifier);

        match reset {
            Some(value) => {
                if *value
                    < SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_millis()
                {
                    return Blocked {
                        blocked: false,
                        reset: 0,
                    };
                }
                Blocked {
                    blocked: true,
                    reset: value.to_owned(),
                }
            }
            None => Blocked {
                blocked: false,
                reset: 0,
            },
        }
    }
    pub fn block_until(&mut self, identifier: &str, reset: u128) {
        self.cache.insert(identifier.to_owned(), reset);
    }
    pub fn set(&mut self, identifier: &str, reset: u128) {
        self.cache.insert(identifier.to_owned(), reset);
    }
    pub fn get(&self, identifier: &str) -> Option<u128> {
        self.cache.get(identifier).copied()
    }
    pub fn incr(&mut self, identifier: &str) {
        let mut value = self.get(identifier).unwrap_or_else(|| 0);
        value += 1;
        self.cache.insert(identifier.to_owned(), value);
    }
}

#[cfg(test)]
mod tests {
    use super::EphemeralCache;

    #[test]
    fn test_get_cache() {
        let key = "ip_address";
        let mut test_cache = EphemeralCache::new();
        test_cache.set(key, 2000);
        assert_eq!(2000, test_cache.get(key).unwrap());
        assert!(test_cache.get("unknown_key").is_none());
    }
}
