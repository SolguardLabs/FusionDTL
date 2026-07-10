use serde::Serialize;

use crate::{FusionError, FusionResult};

pub fn canonical_bytes<T: Serialize>(value: &T) -> FusionResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| FusionError::Serialization(error.to_string()))
}
