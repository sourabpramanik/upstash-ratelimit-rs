//TODO IMPROVE TEST CASES
use common::setup_redis;
mod common;

use nanoid::nanoid;
use upstash_ratelimit_rs::ratelimit::{
	single::{FixedWindow, SlidingWindow, TokenBucket},
	Algorithm, RatelimitConfiguration,
};

#[tokio::test]
async fn test_fixed_window() {
	let redis = setup_redis();
	let client = RatelimitConfiguration::new(redis, true, None);

	let identifier = nanoid!();

	let ratelimit = FixedWindow::new(client, 10, "60s");

	for _ in 1..11 {
		let res = ratelimit.limit(&identifier, None).await;
		assert!(res.success);
	}

	let res = ratelimit.limit(&identifier, None).await;
	assert!(!res.success);
}
#[tokio::test]
async fn test_sliding_window() {
	let redis = setup_redis();

	let client = RatelimitConfiguration::new(redis, true, None);

	let ratelimit = SlidingWindow::new(client, 10, "60s");

	let identifier = nanoid!();

	for _ in 1..11 {
		let res = ratelimit.limit(&identifier, None).await;
		assert!(res.success);
	}

	let res = ratelimit.limit(&identifier, None).await;
	assert!(!res.success);
}

#[tokio::test]
async fn test_token_window() {
	let redis = setup_redis();
	let client = RatelimitConfiguration::new(redis, true, None);

	let ratelimit = TokenBucket::new(client, 10, "30s", 5);

	let identifier = nanoid!();

	for _ in 1..11 {
		let res = ratelimit.limit(&identifier, None).await;
		dbg!(&res);
		assert!(res.success);
	}

	let res = ratelimit.limit(&identifier, None).await;
	assert!(!res.success);
}
