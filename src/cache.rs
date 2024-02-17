use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::common::Blocked;

pub struct Cache {
    cache: HashMap<String, u128>,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            cache: HashMap::new(),
        }
    }

    fn is_blocked(&self, identifier: &str) -> Blocked {
        let reset = self.cache.get(identifier);

        match reset {
            Some(value) => {
                if value.to_owned()
                    > SystemTime::now()
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
