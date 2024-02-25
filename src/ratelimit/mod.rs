pub(crate) mod duration;
pub mod single;
pub mod common;
pub mod cache;

use redis::Client;

use self::{cache::EphemeralCache, common::RatelimitResponse};

pub trait Algorithm{
    async fn limit(&self, identifier: &str) -> RatelimitResponse;
}

#[derive(Debug)]
pub struct RatelimitConfiguration{
    pub redis: Client,
    pub cache: Option<EphemeralCache>,
}

impl RatelimitConfiguration{
    pub fn new(redis: Client, allow_cache: bool) -> Self{
        let mut cache = None;
        
        if allow_cache{
            cache = Some(EphemeralCache::new());
        }

        Self{
            redis,
            cache,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use self::single::FixedWindow;

    use super::*;

    #[tokio::test]
    async fn test_fixed_window() {
        let Ok(redis) = redis::Client::open("redis://default:dbb3xxxxxxxxxxxxxxxxxxxxxxx6f743d@apn1-modern-mollusk-34204.upstash.io:34204") else {
            panic!("Failed to connect")
        };
        let client = RatelimitConfiguration::new(redis, true);

        let ratelimit = FixedWindow::new(client, 10, "60s");

        for _ in 0..10 {
            let res= ratelimit.limit("anonymous").await;
            sleep(Duration::from_millis(1000));
            assert!(res.success);
        }

        let res = ratelimit.limit("anonymous").await;
        assert!(!res.success);
    }
}
