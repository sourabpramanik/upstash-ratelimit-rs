use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use upstash_ratelimit_rs::ratelimit::{single::FixedWindow, Algorithm, RatelimitConfiguration};

#[get("/")]
async fn hello(state: web::Data<AppState>) -> impl Responder {
	let limit_response = state.ratelimit.limit("some-unique-identifier-like-ip", None).await;
	if !limit_response.success {
		return HttpResponse::TooManyRequests().body("Wait for a while");
	}
	HttpResponse::Ok().body("Hello world!")
}

struct AppState {
	ratelimit: FixedWindow,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	dotenv().ok();
	// Create a redis client
	let connection_str = std::env::var("UPSTASH_REDIS_URL").unwrap_or_else(|_| panic!("Expecting UPSTASH_REDIS_URL to be set"));
	let Ok(redis) = redis::Client::open(connection_str) else {
		panic!("Failed to connect")
	};

	// Configure rate limit algorithm
	let client = RatelimitConfiguration::new(redis, false, Some(String::from("my-custom-prefix")));
	let ratelimit = FixedWindow::new(client, 10, "30s");

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(AppState {
				ratelimit: ratelimit.clone(),
			}))
			.service(hello)
	})
	.bind(("127.0.0.1", 8080))?
	.run()
	.await
}
