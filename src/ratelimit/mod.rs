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
