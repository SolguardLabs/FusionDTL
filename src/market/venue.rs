use serde::Serialize;

use crate::{Bps, Digest};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VenueConfig {
    pub venue_digest: Digest,
    pub finality_delay_epochs: u64,
    pub max_relayer_fee_bps: Bps,
    pub active: bool,
}

impl VenueConfig {
    pub fn new(name: &str, finality_delay_epochs: u64, max_relayer_fee_bps: Bps) -> Self {
        Self {
            venue_digest: Digest::from_parts("fusion-venue-v1", &[name.as_bytes()]),
            finality_delay_epochs,
            max_relayer_fee_bps,
            active: true,
        }
    }
}
