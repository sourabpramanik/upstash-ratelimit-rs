use std::future::Future;

use crate::model::{cache::EphemeralCache, common::RateLimitResponse};

pub trait Algorithm {
    type Context;

    fn fixed_window(
        tokens: u32,
        window: &str,
    ) -> Box<dyn Fn(Self::Context, &'static str) -> dyn Future<Output = RateLimitResponse>>;
}
