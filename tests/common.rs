use dotenv::dotenv;
use redis::Client;

pub fn setup_redis() -> Client {
	dotenv().ok();

	let connection_str = std::env::var("UPSTASH_REDIS_URL").unwrap_or_else(|_| panic!("Expecting UPSTASH_REDIS_URL to be set"));
	let Ok(redis) = redis::Client::open(connection_str) else {
		panic!("Failed to connect")
	};

	redis
}
