use crate::model::common::RateLimitResponse;

pub(crate) mod algorithm;
pub(crate) mod duration;
pub mod single;

pub(crate) trait Ratelimit {
    async fn limit(&self, identifier: &str) -> RateLimitResponse;
}
