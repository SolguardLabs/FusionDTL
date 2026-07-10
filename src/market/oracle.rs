use std::collections::BTreeMap;

use serde::Serialize;

use crate::{Amount, AssetId, Bps, Digest, FusionError, FusionResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PriceObservation {
    pub base_asset: AssetId,
    pub quote_asset: AssetId,
    pub numerator: u128,
    pub denominator: u128,
    pub confidence_bps: Bps,
    pub epoch: u64,
    pub digest: Digest,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct OracleBook {
    observations: BTreeMap<Digest, PriceObservation>,
}

impl PriceObservation {
    pub fn new(
        base_asset: AssetId,
        quote_asset: AssetId,
        numerator: u128,
        denominator: u128,
        confidence_bps: Bps,
        epoch: u64,
    ) -> FusionResult<Self> {
        if numerator == 0 || denominator == 0 {
            return Err(FusionError::DivisionByZero);
        }
        let digest = Digest::from_serializable(
            "fusion-price-observation-v1",
            &(
                base_asset,
                quote_asset,
                numerator,
                denominator,
                confidence_bps,
                epoch,
            ),
        )?;
        Ok(Self {
            base_asset,
            quote_asset,
            numerator,
            denominator,
            confidence_bps,
            epoch,
            digest,
        })
    }

    pub fn market_digest(self) -> Digest {
        Digest::from_parts(
            "fusion-price-market-v1",
            &[&self.base_asset.bytes(), &self.quote_asset.bytes()],
        )
    }

    pub fn value_of(self, amount: Amount) -> FusionResult<Amount> {
        amount.checked_mul_ratio(self.numerator, self.denominator)
    }
}

impl OracleBook {
    pub fn publish(&mut self, observation: PriceObservation) {
        self.observations
            .insert(observation.market_digest(), observation);
    }

    pub fn observation(
        &self,
        base_asset: AssetId,
        quote_asset: AssetId,
    ) -> FusionResult<PriceObservation> {
        let market_digest = Digest::from_parts(
            "fusion-price-market-v1",
            &[&base_asset.bytes(), &quote_asset.bytes()],
        );
        self.observations
            .get(&market_digest)
            .copied()
            .ok_or_else(|| FusionError::Policy("price observation unavailable".to_owned()))
    }

    pub fn market_count(&self) -> usize {
        self.observations.len()
    }
}
