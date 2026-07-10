use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, Amount, AssetId, Bps, CellId, Digest, FusionError, FusionResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DeliveryLane {
    pub lane_id: Digest,
    pub source_cell: CellId,
    pub payout_cell: CellId,
    pub asset: AssetId,
    pub min_amount: Amount,
    pub max_amount: Amount,
    pub max_relayer_fee_bps: Bps,
    pub finality_delay_epochs: u64,
    pub enabled: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RelayerQuote {
    pub lane_id: Digest,
    pub relayer: AccountId,
    pub relayer_fee: Amount,
    pub quote_nonce: u64,
    pub expires_at_epoch: u64,
    pub quote_digest: Digest,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct LaneBook {
    lanes: BTreeMap<Digest, DeliveryLane>,
    quotes: BTreeMap<Digest, RelayerQuote>,
}

impl DeliveryLane {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_cell: CellId,
        payout_cell: CellId,
        asset: AssetId,
        min_amount: Amount,
        max_amount: Amount,
        max_relayer_fee_bps: Bps,
        finality_delay_epochs: u64,
        salt: Digest,
    ) -> FusionResult<Self> {
        if min_amount.is_zero() || max_amount.is_zero() {
            return Err(FusionError::ZeroAmount);
        }
        if min_amount > max_amount {
            return Err(FusionError::Policy(
                "lane amount range is invalid".to_owned(),
            ));
        }
        let lane_id = Digest::from_serializable(
            "fusion-delivery-lane-v1",
            &(
                source_cell,
                payout_cell,
                asset,
                min_amount,
                max_amount,
                max_relayer_fee_bps,
                finality_delay_epochs,
                salt,
            ),
        )?;
        Ok(Self {
            lane_id,
            source_cell,
            payout_cell,
            asset,
            min_amount,
            max_amount,
            max_relayer_fee_bps,
            finality_delay_epochs,
            enabled: true,
        })
    }

    pub fn accepts(
        self,
        source_cell: CellId,
        payout_cell: CellId,
        asset: AssetId,
        amount: Amount,
        relayer_fee: Amount,
    ) -> FusionResult<bool> {
        if !self.enabled {
            return Ok(false);
        }
        if self.source_cell != source_cell || self.payout_cell != payout_cell || self.asset != asset
        {
            return Ok(false);
        }
        if amount < self.min_amount || amount > self.max_amount {
            return Ok(false);
        }
        let max_fee = amount.checked_mul_bps(self.max_relayer_fee_bps)?;
        Ok(relayer_fee <= max_fee)
    }
}

impl RelayerQuote {
    pub fn new(
        lane_id: Digest,
        relayer: AccountId,
        relayer_fee: Amount,
        quote_nonce: u64,
        expires_at_epoch: u64,
    ) -> FusionResult<Self> {
        let quote_digest = Digest::from_serializable(
            "fusion-relayer-quote-v1",
            &(lane_id, relayer, relayer_fee, quote_nonce, expires_at_epoch),
        )?;
        Ok(Self {
            lane_id,
            relayer,
            relayer_fee,
            quote_nonce,
            expires_at_epoch,
            quote_digest,
        })
    }
}

impl LaneBook {
    pub fn register_lane(&mut self, lane: DeliveryLane) -> FusionResult<Digest> {
        if self.lanes.contains_key(&lane.lane_id) {
            return Err(FusionError::Policy(
                "delivery lane already exists".to_owned(),
            ));
        }
        self.lanes.insert(lane.lane_id, lane);
        Ok(lane.lane_id)
    }

    pub fn submit_quote(&mut self, quote: RelayerQuote) -> FusionResult<Digest> {
        if !self.lanes.contains_key(&quote.lane_id) {
            return Err(FusionError::Policy("delivery lane not found".to_owned()));
        }
        self.quotes.insert(quote.quote_digest, quote);
        Ok(quote.quote_digest)
    }

    pub fn resolve_lane(
        &self,
        source_cell: CellId,
        payout_cell: CellId,
        asset: AssetId,
        amount: Amount,
        relayer_fee: Amount,
    ) -> FusionResult<DeliveryLane> {
        for lane in self.lanes.values().copied() {
            if lane.accepts(source_cell, payout_cell, asset, amount, relayer_fee)? {
                return Ok(lane);
            }
        }
        Err(FusionError::Policy(
            "delivery lane not available".to_owned(),
        ))
    }

    pub fn lane_count(&self) -> usize {
        self.lanes.len()
    }

    pub fn quote_count(&self) -> usize {
        self.quotes.len()
    }
}
