# Unofficial Upstash rate limit SDK for Rust
A rate limiting SDK built for Rust ecosystem that uses in-memory data storage.

## Inspiration
This rate limit SDK is inspired from the official [TypeScript rate limit SDK](https://github.com/upstash/ratelimit) created by [Upstash](https://upstash.com) team. 

## Setup

1) To setup the ratelimiter, first create a client instance of Redis that can store the request counts for given set window:
```rust
	let connection_str = std::env::var("UPSTASH_REDIS_URL").unwrap_or_else(|_| panic!("Expecting UPSTASH_REDIS_URL to be set"));
	let Ok(redis) = redis::Client::open(connection_str) else {
		panic!("Failed to connect")
	};
```
2) Create a client instance of the `RatelimitConfiguration` using the Redis client:

3) Use the client configuration to create new instance of any one of the three rate limiting algorithms:

For example: Using the fixed window algorithm to limit 10 requests in 30 seconds of window.

```rust
	let client = RatelimitConfiguration::new(redis, true, Some(String::from("my-custom-prefix")));
	let ratelimit = FixedWindow::new(client, 10, "30s");
```
> In then above client configuration, using the Ephemerale cache to avoid making Redis calls if the request is already blocked and adding a custom prefix string will override the default prefix string, 

Use the `ratelimit` instance to call the limit function in the any request calls to rate limit your requests:
```rust
let limit_response = state.ratelimit.limit("some-unique-identifier-like-ip", None).await;
```

## Roadmap
Coming soon...

## Contributions

This library is still in WIP. Very soon all the contributions will be accepted.
