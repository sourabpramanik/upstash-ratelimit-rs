#[derive(Debug)]
pub struct RateLimitResponse {
    pub success: bool,
    pub limit: u32,
    pub remaining: u32,
    pub reset: u128,
}
