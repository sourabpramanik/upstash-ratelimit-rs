use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::common::Blocked;

/**
 * EphemeralCache is used to block certain identifiers right away in case they have already exceeded the ratelimit.
 */
pub struct EphemeralCache {
    cache: HashMap<String, u128>,
}

impl EphemeralCache {
    pub fn new() -> Self {
        EphemeralCache {
            cache: HashMap::new(),
        }
    }

    fn is_blocked(&self, identifier: &str) -> Blocked {
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

    fn block_until(&mut self, identifier: &str, reset: u128) {
        self.cache.insert(identifier.to_owned(), reset);
    }

    fn set(&mut self, identifier: &str, reset: u128) {
        self.cache.insert(identifier.to_owned(), reset);
    }

    fn get(&self, identifier: &str) -> Option<u128> {
        self.cache.get(identifier).copied()
    }

    fn incr(&mut self, identifier: &str) {
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

    #[test]
    fn test_incr_cache() {
        let key_one = "ip_address_one";
        let key_two = "ip_address_two";

        let mut test_cache = EphemeralCache::new();

        test_cache.incr(key_one);

        assert_eq!(1, test_cache.get(key_one).unwrap());

        test_cache.set(key_two, 4000);
        test_cache.incr(key_two);

        assert_ne!(4000, test_cache.get(key_two).unwrap())
    }

    #[test]
    fn test_is_blocked() {
        let mut test_cache = EphemeralCache::new();
        let key = "ip_address";

        assert!(!test_cache.is_blocked(key).blocked);

        test_cache.block_until(key, 1708223649);
        assert!(!test_cache.is_blocked(key).blocked);
    }
}
