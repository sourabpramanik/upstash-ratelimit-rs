use redis::Client;

use super::cache::EphemeralCache;

#[derive(Debug)]
pub struct SingleRegionContext {
    pub redis: Client,
    pub cache: Option<EphemeralCache>,
}

#[derive(Debug)]
pub(crate) struct MultiRegionContext {
    redis: Vec<Client>,
    cache: Option<EphemeralCache>,
}

#[derive(Debug)]
pub enum RegionContext {
    SingleRegion,
    MultiRegion,
}
