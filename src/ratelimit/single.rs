use std::{
	cmp::max,
	time::{SystemTime, UNIX_EPOCH},
};

use super::{duration::into_milliseconds, Algorithm, RatelimitConfiguration};
use crate::ratelimit::RatelimitResponse;

#[derive(Debug, Clone)]
pub struct FixedWindow {
	client: RatelimitConfiguration,
	tokens: u32,
	duration: u128,
}

impl FixedWindow {
	pub fn new(client: RatelimitConfiguration, tokens: u32, window: &str) -> Self {
		Self {
			client,
			tokens,
			duration: into_milliseconds(window),
		}
	}
}

impl Algorithm for FixedWindow {
	async fn limit(&self, identifier: &str, rate: Option<u32>) -> RatelimitResponse {
		let tokens = self.tokens;
		let duration = self.duration;

		let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
			panic!("Unable to get current time");
		};
		let bucket = now.as_millis() / duration;
		let key = [&self.client.prefix, identifier, bucket.to_string().as_str()].join(":");

		if self.client.cache.is_some() && self.client.cache.clone().unwrap().is_blocked(identifier).blocked {
			return RatelimitResponse {
				success: false,
				limit: tokens,
				remaining: 0,
				reset: 0,
			};
		}

		let mut connection = self.client.redis.get_async_connection().await.unwrap();

		let script = redis::Script::new(include_str!("../../scripts/single_region/fixed_window.lua"));

		let increment_by = rate.unwrap_or(1);

		let result: Result<i32, redis::RedisError> = script
			.key(key)
			.arg(vec![duration as u64, increment_by as u64])
			.invoke_async(&mut connection)
			.await;

		let used_tokens: i32 = match result {
			Ok(val) => val,
			Err(err) => {
				println!("Failed to evaluate: {}", err);
				return RatelimitResponse {
					success: false,
					limit: tokens,
					remaining: 0,
					reset: 0,
				};
			}
		};

		let success = used_tokens <= tokens as i32;
		let reset = (bucket + 1) * duration;
		let remaining = max(0, tokens as i32 - used_tokens) as u32;

		if self.client.cache.is_some() && !success {
			self.client.cache.clone().unwrap().block_until(identifier, reset)
		}
		RatelimitResponse {
			success,
			limit: tokens,
			remaining,
			reset,
		}
	}
}

#[derive(Debug, Clone)]
pub struct SlidingWindow {
	client: RatelimitConfiguration,
	tokens: u32,
	duration: u128,
}

impl SlidingWindow {
	pub fn new(client: RatelimitConfiguration, tokens: u32, window: &str) -> Self {
		Self {
			client,
			tokens,
			duration: into_milliseconds(window),
		}
	}
}

impl Algorithm for SlidingWindow {
	async fn limit(&self, identifier: &str, rate: Option<u32>) -> RatelimitResponse {
		let tokens = self.tokens;
		let duration = self.duration;

		let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
			panic!("Unable to get current time");
		};

		let current_window = now.as_millis() / duration;
		let current_key = [&self.client.prefix, identifier, current_window.to_string().as_str()].join(":");

		let previous_widow = current_window - 1;
		let previous_key = [&self.client.prefix, identifier, previous_widow.to_string().as_str()].join(":");

		if self.client.cache.is_some() && self.client.cache.clone().unwrap().is_blocked(identifier).blocked {
			return RatelimitResponse {
				success: false,
				limit: tokens,
				remaining: 0,
				reset: 0,
			};
		}

		let mut connection = self.client.redis.get_async_connection().await.unwrap();

		let script = redis::Script::new(include_str!("../../scripts/single_region/sliding_window.lua"));

		let increment_by = rate.unwrap_or(1);

		let result: Result<i32, redis::RedisError> = script
			.key(vec![current_key, previous_key])
			.arg(vec![tokens, now.as_millis() as u32, duration as u32, increment_by])
			.invoke_async(&mut connection)
			.await;

		let remaining_tokens: i32 = match result {
			Ok(val) => val,
			Err(err) => {
				println!("Failed to evaluate: {}", err);
				return RatelimitResponse {
					success: false,
					limit: tokens,
					remaining: 0,
					reset: 0,
				};
			}
		};
		let success = remaining_tokens >= 0;
		let reset = (current_window + 1) * duration;
		let remaining = max(0, remaining_tokens) as u32;

		if self.client.cache.is_some() && !success {
			self.client.cache.clone().unwrap().block_until(identifier, reset)
		}
		RatelimitResponse {
			success,
			limit: tokens,
			remaining,
			reset,
		}
	}
}

#[derive(Debug, Clone)]
pub struct TokenBucket {
	client: RatelimitConfiguration,
	tokens: u32,
	interval: u128,
	refill_rate: u32,
}

impl TokenBucket {
	pub fn new(client: RatelimitConfiguration, tokens: u32, interval: &str, refill_rate: u32) -> Self {
		Self {
			client,
			tokens,
			interval: into_milliseconds(interval),
			refill_rate,
		}
	}
}

impl Algorithm for TokenBucket {
	async fn limit(&self, identifier: &str, rate: Option<u32>) -> RatelimitResponse {
		let tokens = self.tokens;
		let interval = self.interval;
		if self.client.cache.is_some() && self.client.cache.clone().unwrap().is_blocked(identifier).blocked {
			return RatelimitResponse {
				success: false,
				limit: tokens,
				remaining: 0,
				reset: 0,
			};
		}

		let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
			panic!("Unable to get current time");
		};
		let key = [&self.client.prefix, identifier].join(":");
		let mut connection = self.client.redis.get_async_connection().await.unwrap();
		let script = redis::Script::new(include_str!("../../scripts/single_region/token_window.lua"));
		let increment_by = rate.unwrap_or(1);

		let result: Result<(i32, i32), redis::RedisError> = script
			.key(key)
			.arg(vec![
				tokens,
				interval as u32,
				self.refill_rate,
				(now.as_millis() / interval) as u32,
				increment_by,
			])
			.invoke_async(&mut connection)
			.await;

		let (remaining, reset) = match result {
			Ok(val) => val,
			Err(err) => {
				println!("Failed to evaluate: {}", err);
				return RatelimitResponse {
					success: false,
					limit: tokens,
					remaining: 0,
					reset: 0,
				};
			}
		};

		let success = remaining > 0;

		if self.client.cache.is_some() && !success {
			self.client.cache.clone().unwrap().block_until(identifier, reset as u128)
		}
		RatelimitResponse {
			success,
			limit: tokens,
			remaining: remaining as u32,
			reset: reset as u128,
		}
	}
}
