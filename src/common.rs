use redis::Client;

use crate::cache::Cache;

#[derive(Debug)]
pub(crate) struct Blocked {
    pub blocked: bool,
    pub reset: u128,
}

pub struct RateLimitResponse {
    pub success: bool,
    pub limit: u32,
    pub remaining: u32,
    pub reset: u32,
}

pub struct RegionContext {
    pub redis: Client,
    pub cache: Cache,
}
