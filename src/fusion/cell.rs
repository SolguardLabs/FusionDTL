use serde::Serialize;

use crate::{
    AccountId, Amount, AssetId, Bps, CellId, Digest, FusionError, FusionResult, ReceiptId,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CellConfig {
    pub cell_id: CellId,
    pub controller: AccountId,
    pub reserve_asset: AssetId,
    pub lane: u16,
    pub reserve_floor: Amount,
    pub max_pending_bps: Bps,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LiquidityCell {
    pub config: CellConfig,
    pub reserve_balance: Amount,
    pub pending_liability: Amount,
    pub issued_receipts: u64,
    pub settled_receipts: u64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReceiptRecord {
    pub receipt_id: ReceiptId,
    pub source_cell: CellId,
    pub issuer: AccountId,
    pub settled: bool,
}

impl CellConfig {
    pub fn new(
        controller: AccountId,
        reserve_asset: AssetId,
        lane: u16,
        reserve_floor: Amount,
        max_pending_bps: Bps,
        salt: Digest,
    ) -> Self {
        Self {
            cell_id: CellId::derive(controller, reserve_asset, lane, salt),
            controller,
            reserve_asset,
            lane,
            reserve_floor,
            max_pending_bps,
        }
    }
}

impl LiquidityCell {
    pub fn new(config: CellConfig) -> Self {
        Self {
            config,
            reserve_balance: Amount::zero(),
            pending_liability: Amount::zero(),
            issued_receipts: 0,
            settled_receipts: 0,
        }
    }

    pub fn deposit(&mut self, amount: Amount) -> FusionResult<()> {
        if amount.is_zero() {
            return Err(FusionError::ZeroAmount);
        }
        self.reserve_balance = self.reserve_balance.checked_add(amount)?;
        Ok(())
    }

    pub fn register_receipt(&mut self, amount: Amount) -> FusionResult<()> {
        let next_pending = self.pending_liability.checked_add(amount)?;
        if !self.pending_within_limit(next_pending)? {
            return Err(FusionError::Policy(
                "cell pending liability exceeds limit".to_owned(),
            ));
        }
        self.pending_liability = next_pending;
        self.issued_receipts = self
            .issued_receipts
            .checked_add(1)
            .ok_or(FusionError::AmountOverflow)?;
        Ok(())
    }

    pub fn settle_liability(&mut self, amount: Amount) -> FusionResult<()> {
        self.pending_liability = self.pending_liability.checked_sub(amount)?;
        self.settled_receipts = self
            .settled_receipts
            .checked_add(1)
            .ok_or(FusionError::AmountOverflow)?;
        Ok(())
    }

    pub fn pay_delivery(&mut self, amount: Amount) -> FusionResult<()> {
        let next_reserve = self.reserve_balance.checked_sub(amount)?;
        if next_reserve < self.config.reserve_floor {
            return Err(FusionError::Policy("cell reserve floor reached".to_owned()));
        }
        self.reserve_balance = next_reserve;
        Ok(())
    }

    pub fn surplus(&self) -> FusionResult<Amount> {
        let required = self
            .pending_liability
            .checked_add(self.config.reserve_floor)?;
        if self.reserve_balance <= required {
            return Ok(Amount::zero());
        }
        self.reserve_balance.checked_sub(required)
    }

    pub fn sweep(&mut self, amount: Amount) -> FusionResult<()> {
        if amount > self.surplus()? {
            return Err(FusionError::Policy(
                "cell surplus is below requested amount".to_owned(),
            ));
        }
        self.reserve_balance = self.reserve_balance.checked_sub(amount)?;
        Ok(())
    }

    pub fn liquidity_digest(&self) -> FusionResult<Digest> {
        Digest::from_serializable(
            "fusion-cell-liquidity-v1",
            &(
                self.config.cell_id,
                self.reserve_balance,
                self.pending_liability,
                self.issued_receipts,
                self.settled_receipts,
            ),
        )
    }

    fn pending_within_limit(&self, next_pending: Amount) -> FusionResult<bool> {
        if self.reserve_balance.is_zero() {
            return Ok(false);
        }
        let ratio = next_pending
            .checked_mul(10_000)?
            .checked_div(self.reserve_balance.units())?;
        Ok(ratio.units() <= u128::from(self.config.max_pending_bps.units()))
    }
}
