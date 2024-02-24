use std::{
    cmp::max, time::{SystemTime, UNIX_EPOCH}
};

use redis::{Client, AsyncCommands};

use super::duration::into_milliseconds;
use crate::model::{common::{Algorithm, AlgorithmResponse, RateLimitResponse}, region::SingleRegionContext};

#[derive(Debug)]
pub struct SingleRegionConfig {
    redis: Client,
    prefix: Option<String>,
}

#[derive(Debug)]
pub struct SingleRegionRateLimit<T> {
    ctx: T,
    prefix: String,
}

impl<T> Algorithm for SingleRegionRateLimit<T>{
    type TContext = SingleRegionContext;
    fn fixed_window(
        tokens: u32,
        window: &str,
    ) -> AlgorithmResponse<Self::TContext> {
        let window_duration = into_milliseconds(window);

        Box::new(move |ctx: Self::TContext, identifier: String| {
            Box::pin(async move {
                let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
                    panic!("Unable to get current time");
                };
                let bucket = now.as_millis() / window_duration;
                let key = vec![&identifier, bucket.to_string().as_str()].join(":");
                
                if ctx.cache.is_some() {
                    if ctx.cache.unwrap().is_blocked(&identifier).blocked {
                        return RateLimitResponse {
                            success: false,
                            limit: tokens,
                            remaining: 0,
                            reset: 0
                        };
                    }
                }

                let mut connection = ctx.redis.get_async_connection().await.unwrap();
                
                // Script taken from @upstash/ratelimit
                let script = redis::Script::new(
                    r"
                    local key     = KEYS[1]
                    local window  = ARGV[1]
                    
                    local value = redis.call('INCR', key)
                    if value == 1 then 
                    -- The first time this key is set, the value will be 1.
                    -- So we only need the expire command once
                    redis.call('PEXPIRE', key, window)
                    end
                    
                    return value",
                );

                let result: Result<u32, redis::RedisError> = script
                    .key(key)
                    .arg(window_duration as u64)
                    .invoke_async(&mut connection).await;

                let used_tokens: u32 = match result {
                    Ok(val) => val,
                    Err(err) => {
                        println!("Failed to evaluate: {}", err);
                        return RateLimitResponse{
                            success: false,
                            limit: tokens,
                            remaining: 0,
                            reset: 0
                        } 
                    }
                };

                let success = used_tokens <= tokens;
                let reset = (bucket + 1) * window_duration;
                let remaining = max(0, tokens - used_tokens);
                RateLimitResponse {
                    success,
                    limit: tokens,
                    remaining,
                    reset,
                }
            })
        })
    }
}
