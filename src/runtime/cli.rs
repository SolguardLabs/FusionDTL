use crate::{FusionError, FusionResult};

pub fn run() -> FusionResult<()> {
    let scenario = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "settle".to_owned());
    let report = super::scenarios::run_named(&scenario)?;
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| FusionError::Serialization(error.to_string()))?;
    println!("{json}");
    Ok(())
}
