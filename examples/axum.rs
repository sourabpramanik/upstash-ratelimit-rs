use axum::{extract::State, routing::get, Router};
use dotenv::dotenv;
use upstash_ratelimit_rs::ratelimit::{single::FixedWindow, Algorithm, RatelimitConfiguration};

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();
	dotenv().ok();

	// Create a redis client
	let connection_str = std::env::var("UPSTASH_REDIS_URL").unwrap_or_else(|_| panic!("Expecting UPSTASH_REDIS_URL to be set"));
	let Ok(redis) = redis::Client::open(connection_str) else {
		panic!("Failed to connect")
	};

	// Configure rate limit algorithm
	let client = RatelimitConfiguration::new(redis, false, Some(String::from("my-custom-prefix")));
	let ratelimit = FixedWindow::new(client, 10, "30s");

	let app = Router::new().route("/", get(root)).with_state(ratelimit.clone());

	let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

async fn root(State(ratelimit): State<FixedWindow>) -> &'static str {
	let limit_response = ratelimit.limit("some-unique-identifier-like-ip", None).await;
	if !limit_response.success {
		return "Wait for a while";
	}
	"Hello, World!"
}
