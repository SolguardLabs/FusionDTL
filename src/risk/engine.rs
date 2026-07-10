use serde::Serialize;

use crate::{Amount, Bps, CellId, FusionError, FusionResult, LiquidityCell, PacketId};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RiskLimits {
    pub current_epoch: u64,
    pub max_receipt_amount: Amount,
    pub max_relayer_fee_bps: Bps,
    pub min_cell_reserve_after_delivery: Amount,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RiskSnapshot {
    pub packet_id: PacketId,
    pub cell_id: CellId,
    pub amount: Amount,
    pub relayer_fee: Amount,
    pub projected_reserve: Amount,
    pub accepted: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct RiskEngine {
    limits: RiskLimits,
}

impl RiskLimits {
    pub fn new(
        current_epoch: u64,
        max_receipt_amount: Amount,
        max_relayer_fee_bps: Bps,
        min_cell_reserve_after_delivery: Amount,
    ) -> FusionResult<Self> {
        if max_receipt_amount.is_zero() {
            return Err(FusionError::ZeroAmount);
        }
        Ok(Self {
            current_epoch,
            max_receipt_amount,
            max_relayer_fee_bps,
            min_cell_reserve_after_delivery,
        })
    }
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            current_epoch: 900,
            max_receipt_amount: Amount::new(50_000_000_000).expect("valid amount"),
            max_relayer_fee_bps: Bps::new(75).expect("valid bps"),
            min_cell_reserve_after_delivery: Amount::new(5_000_000_000).expect("valid amount"),
        }
    }
}

impl RiskEngine {
    pub fn new(limits: RiskLimits) -> Self {
        Self { limits }
    }

    pub const fn limits(&self) -> RiskLimits {
        self.limits
    }

    pub fn set_limits(&mut self, limits: RiskLimits) {
        self.limits = limits;
    }

    pub fn evaluate_delivery(
        &self,
        packet_id: PacketId,
        cell: &LiquidityCell,
        amount: Amount,
        relayer_fee: Amount,
    ) -> FusionResult<RiskSnapshot> {
        if amount > self.limits.max_receipt_amount {
            return Err(FusionError::Policy(
                "receipt amount exceeds limit".to_owned(),
            ));
        }
        let allowed_fee = amount.checked_mul_bps(self.limits.max_relayer_fee_bps)?;
        if relayer_fee > allowed_fee {
            return Err(FusionError::Policy("relayer fee exceeds limit".to_owned()));
        }
        let projected_reserve = cell.reserve_balance.checked_sub(amount)?;
        if projected_reserve < self.limits.min_cell_reserve_after_delivery {
            return Err(FusionError::Policy(
                "projected reserve below protocol floor".to_owned(),
            ));
        }
        Ok(RiskSnapshot {
            packet_id,
            cell_id: cell.config.cell_id,
            amount,
            relayer_fee,
            projected_reserve,
            accepted: true,
        })
    }
}

impl Default for RiskEngine {
    fn default() -> Self {
        Self::new(RiskLimits::default())
    }
}
