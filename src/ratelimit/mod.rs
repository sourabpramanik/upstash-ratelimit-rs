pub(crate) mod duration;
pub mod single;
pub mod common;
pub mod cache;

use redis::Client;

use self::{cache::EphemeralCache, common::RatelimitResponse};

pub trait Algorithm{
    fn limit(&self, identifier: &str, rate: Option<u32>) -> impl std::future::Future<Output = RatelimitResponse> + Send;
}

#[derive(Debug, Clone)]
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

    use self::single::{FixedWindow, SlidingWindow};

    use super::*;
    use dotenv::dotenv;

    #[tokio::test]
    async fn test_fixed_window() {
        dotenv().ok();

        let connection_str = std::env::var("UPSTASH_REDIS_URL").unwrap_or_else(|_|panic!("Expecting UPSTASH_REDIS_URL to be set"));
        let Ok(redis) = redis::Client::open(connection_str) else {
            panic!("Failed to connect")
        };
        let client = RatelimitConfiguration::new(redis, true);
        
        let identifier = "anonymous32";
            
        let ratelimit = FixedWindow::new(client, 10, "60s");

        for _ in 1..11 {
            let res= ratelimit.limit(identifier, None).await;
            
            assert!(res.success);
        }

        let res = ratelimit.limit(identifier, None).await;
        assert!(!res.success);
    }
    #[tokio::test]
    async fn test_sliding_window() {
        dotenv().ok();

        let connection_str = std::env::var("UPSTASH_REDIS_URL").unwrap_or_else(|_|panic!("Expecting UPSTASH_REDIS_URL to be set"));
        let Ok(redis) = redis::Client::open(connection_str) else {
            panic!("Failed to connect")
        };
        let client = RatelimitConfiguration::new(redis, true);

        let ratelimit = SlidingWindow::new(client, 10, "60s");

        let identifier = "anonymous12";

        for _ in 1..11 {
            let res= ratelimit.limit(identifier, None).await;
            assert!(res.success);
        }

        let res = ratelimit.limit(identifier, None).await;
        assert!(!res.success);
    }

}
