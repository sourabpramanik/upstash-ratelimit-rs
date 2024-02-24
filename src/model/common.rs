use std::{future::Future, pin::Pin};

use super::region::SingleRegionContext;

#[derive(Debug)]
pub struct RateLimitResponse {
    pub success: bool,
    pub limit: u32,
    pub remaining: u32,
    pub reset: u128,
}

pub type AlgorithmResponse =  Box<dyn Fn(SingleRegionContext, String) -> Pin<Box<dyn Future<Output = RateLimitResponse> + Send>>>;
