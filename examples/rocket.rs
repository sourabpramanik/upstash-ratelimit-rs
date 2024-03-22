#[macro_use]
extern crate rocket;

use dotenv::dotenv;
use rocket::http::Status;
use rocket::response::{content, status};
use rocket::State;
use upstash_ratelimit_rs::ratelimit::{single::FixedWindow, Algorithm, RatelimitConfiguration};

struct AppState {
	ratelimit: FixedWindow,
}

#[get("/")]
async fn index(state: &State<AppState>) -> status::Custom<content::RawJson<&'static str>> {
	let limit_response = state.ratelimit.limit("some-unique-identifier-like-ip", None).await;
	if !limit_response.success {
		return status::Custom(Status::TooManyRequests, content::RawJson("{ \"message\": \"Wait for a while\" }"));
	}
	status::Custom(Status::Ok, content::RawJson("{ \"message\": \"Hello World!\" }"))
}

#[launch]
fn rocket() -> _ {
	dotenv().ok();
	// Create a redis client
	let connection_str = std::env::var("UPSTASH_REDIS_URL").unwrap_or_else(|_| panic!("Expecting UPSTASH_REDIS_URL to be set"));
	let Ok(redis) = redis::Client::open(connection_str) else {
		panic!("Failed to connect")
	};

	// Configure rate limit algorithm
	let client = RatelimitConfiguration::new(redis, false, Some(String::from("my-custom-prefix")));
	let ratelimit = FixedWindow::new(client, 10, "30s");

	rocket::build().mount("/", routes![index]).manage(AppState {
		ratelimit: ratelimit.clone(),
	})
}
