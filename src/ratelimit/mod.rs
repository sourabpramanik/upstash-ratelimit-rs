pub mod cache;
pub mod common;
pub(crate) mod duration;
pub mod single;

use redis::Client;

use self::{cache::EphemeralCache, common::RatelimitResponse};

pub trait Algorithm {
	fn limit(&self, identifier: &str, rate: Option<u32>) -> impl std::future::Future<Output = RatelimitResponse> + Send;
}

#[derive(Debug, Clone)]
pub struct RatelimitConfiguration {
	pub(crate) redis: Client,
	pub(crate) cache: Option<EphemeralCache>,
	pub(crate) prefix: String,
}

impl RatelimitConfiguration {
	pub fn new(redis: Client, allow_cache: bool, prefix_str: Option<String>) -> Self {
		let mut cache = None;
		let mut prefix = String::from("@upstash/ratelimit");

		if allow_cache {
			cache = Some(EphemeralCache::new());
		}

		if prefix_str.is_some() {
			prefix = prefix_str.unwrap();
		}

		Self { redis, cache, prefix }
	}
}

#[cfg(test)]
mod tests {

	use self::single::{FixedWindow, SlidingWindow, TokenBucket};

	use super::*;
	use dotenv::dotenv;

	#[tokio::test]
	async fn test_fixed_window() {
		dotenv().ok();

		let connection_str = std::env::var("UPSTASH_REDIS_URL").unwrap_or_else(|_| panic!("Expecting UPSTASH_REDIS_URL to be set"));
		let Ok(redis) = redis::Client::open(connection_str) else {
			panic!("Failed to connect")
		};
		let client = RatelimitConfiguration::new(redis, true, None);

		let identifier = "anonymous32";

		let ratelimit = FixedWindow::new(client, 10, "60s");

		for _ in 1..11 {
			let res = ratelimit.limit(identifier, None).await;

			assert!(res.success);
		}

		let res = ratelimit.limit(identifier, None).await;
		assert!(!res.success);
	}
	#[tokio::test]
	async fn test_sliding_window() {
		dotenv().ok();

		let connection_str = std::env::var("UPSTASH_REDIS_URL").unwrap_or_else(|_| panic!("Expecting UPSTASH_REDIS_URL to be set"));
		let Ok(redis) = redis::Client::open(connection_str) else {
			panic!("Failed to connect")
		};
		let client = RatelimitConfiguration::new(redis, true, None);

		let ratelimit = SlidingWindow::new(client, 10, "60s");

		let identifier = "anonymous12";

		for _ in 1..11 {
			let res = ratelimit.limit(identifier, None).await;
			assert!(res.success);
		}

		let res = ratelimit.limit(identifier, None).await;
		assert!(!res.success);
	}

	#[tokio::test]
	async fn test_token_window() {
		dotenv().ok();

		let connection_str = std::env::var("UPSTASH_REDIS_URL").unwrap_or_else(|_| panic!("Expecting UPSTASH_REDIS_URL to be set"));
		let Ok(redis) = redis::Client::open(connection_str) else {
			panic!("Failed to connect")
		};
		let client = RatelimitConfiguration::new(redis, true, None);

		let ratelimit = TokenBucket::new(client, 10, "10s", 5);

		let identifier = "anonymous52";

		for _ in 1..11 {
			let res = ratelimit.limit(identifier, None).await;
			assert!(res.success);
		}

		let res = ratelimit.limit(identifier, None).await;
		assert!(!res.success);
	}
}
