use std::{
    cmp::max, time::{SystemTime, UNIX_EPOCH}
};
use redis::{Client, AsyncCommands};

use super::{duration::into_milliseconds, RatelimitConfiguration, Algorithm};
use crate::ratelimit::RatelimitResponse;

#[derive(Debug)]
pub struct FixedWindow{
    client: RatelimitConfiguration,
    tokens: u32,
    duration: u128,
}

impl FixedWindow{
    pub fn new(client: RatelimitConfiguration, tokens: u32, window: &str) -> Self{
        Self{
            client,
            tokens,
            duration: into_milliseconds(window),
        }
    }
}

 impl Algorithm for FixedWindow{
    async fn limit(&self, identifier: &str) -> RatelimitResponse{
        let tokens = self.tokens;
        let duration = self.duration;

        let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
            panic!("Unable to get current time");
        };
        let bucket = now.as_millis() / duration;
        let key = vec![&identifier, bucket.to_string().as_str()].join(":");
        
        if self.client.cache.is_some() {
            if self.client.cache.clone().unwrap().is_blocked(&identifier).blocked {
                return RatelimitResponse {
                    success: false,
                    limit: tokens,
                    remaining: 0,
                    reset: 0
                };
            }
        }

        let mut connection = self.client.redis.get_async_connection().await.unwrap();
        
        let script = redis::Script::new(include_str!("../../scripts/single_region/fixed_window.lua"));

        let result: Result<i32, redis::RedisError> = script
            .key(key)
            .arg(duration as u64)
            .invoke_async(&mut connection).await;

        let used_tokens: i32 = match result {
            Ok(val) => val,
            Err(err) => {
                println!("Failed to evaluate: {}", err);
                return RatelimitResponse{
                    success: false,
                    limit: tokens,
                    remaining: 0,
                    reset: 0
                } 
            }
        };
   
        let success = used_tokens <= tokens as i32;
        let reset = (bucket + 1) * duration;
        let remaining = max(0, tokens as i32 - used_tokens) as u32;

        if self.client.cache.is_some() && !success{
            self.client.cache.clone().unwrap().block_until(&identifier, reset)
        }
        RatelimitResponse {
            success,
            limit: tokens,
            remaining,
            reset,
        }
    }
}

#[derive(Debug)]
pub struct SlidingWindow{
    client: RatelimitConfiguration,
    tokens: u32,
    duration: u128,
}

impl SlidingWindow{
    pub fn new(client: RatelimitConfiguration, tokens: u32, window: &str) -> Self{
        Self{
            client,
            tokens,
            duration: into_milliseconds(window),
        }
    }
}

 impl Algorithm for SlidingWindow{
    async fn limit(&self, identifier: &str) -> RatelimitResponse{
        let tokens = self.tokens;
        let duration = self.duration;

        let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
            panic!("Unable to get current time");
        };
        
        let current_window = now.as_millis() / duration;
        let current_key = vec![&identifier, current_window.to_string().as_str()].join(":");

        let previous_widow = current_window - 1;
        let previous_key = vec![&identifier, previous_widow.to_string().as_str()].join(":");
        
        if self.client.cache.is_some() {
            if self.client.cache.clone().unwrap().is_blocked(&identifier).blocked {
                return RatelimitResponse {
                    success: false,
                    limit: tokens,
                    remaining: 0,
                    reset: 0
                };
            }
        }

        let mut connection = self.client.redis.get_async_connection().await.unwrap();
        
        let script = redis::Script::new(include_str!("../../scripts/single_region/sliding_window.lua"));

        let result: Result<i32, redis::RedisError> = script
            .key(vec![current_key, previous_key])
            .arg(vec![
                tokens,
                now.as_millis() as u32,
                duration as u32,
                1,
            ])
            .invoke_async(&mut connection).await;

        let remaining_tokens: i32 = match result {
            Ok(val) => val,
            Err(err) => {
                println!("Failed to evaluate: {}", err);
                return RatelimitResponse{
                    success: false,
                    limit: tokens,
                    remaining: 0,
                    reset: 0
                } 
            }
        };
        let success = remaining_tokens >= 0;
        let reset = (current_window + 1) * duration;
        let remaining = max(0, remaining_tokens) as u32;

        if self.client.cache.is_some() && !success{
            self.client.cache.clone().unwrap().block_until(&identifier, reset)
        }
        RatelimitResponse {
            success,
            limit: tokens,
            remaining,
            reset,
        }
    }
}
