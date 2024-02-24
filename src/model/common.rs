use std::{future::Future, pin::Pin};

#[derive(Debug)]
pub struct RateLimitResponse {
    pub success: bool,
    pub limit: u32,
    pub remaining: u32,
    pub reset: u128,
}

pub trait Algorithm {
    type TContext;
   
    fn fixed_window(tokens: u32, window: &str) -> AlgorithmResponse<Self::TContext>;
}

pub type AlgorithmResponse<T> =  Box<dyn Fn(T, String) -> Pin<Box<dyn Future<Output = RateLimitResponse> + Send>>>;
