use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, Amount, AssetId, Bps, FusionError, FusionResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FeeSchedule {
    pub treasury: AccountId,
    pub protocol_fee_bps: Bps,
    pub insurance_fee_bps: Bps,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InsuranceReserve {
    pub asset: AssetId,
    pub balance: Amount,
    pub floor: Amount,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct TreasuryBook {
    schedule: Option<FeeSchedule>,
    insurance: Option<InsuranceReserve>,
    accrued: BTreeMap<AssetId, Amount>,
}

impl FeeSchedule {
    pub fn new(
        treasury: AccountId,
        protocol_fee_bps: Bps,
        insurance_fee_bps: Bps,
    ) -> FusionResult<Self> {
        let total = u32::from(protocol_fee_bps.units()) + u32::from(insurance_fee_bps.units());
        if total > 10_000 {
            return Err(FusionError::Policy("fee schedule exceeds range".to_owned()));
        }
        Ok(Self {
            treasury,
            protocol_fee_bps,
            insurance_fee_bps,
        })
    }
}

impl InsuranceReserve {
    pub fn new(asset: AssetId, balance: Amount, floor: Amount) -> FusionResult<Self> {
        if floor > balance {
            return Err(FusionError::Policy(
                "insurance floor exceeds balance".to_owned(),
            ));
        }
        Ok(Self {
            asset,
            balance,
            floor,
        })
    }
}

impl TreasuryBook {
    pub fn configure_schedule(&mut self, schedule: FeeSchedule) {
        self.schedule = Some(schedule);
    }

    pub fn configure_insurance(&mut self, reserve: InsuranceReserve) {
        self.insurance = Some(reserve);
    }

    pub fn protocol_fee(&self, notional: Amount) -> FusionResult<Amount> {
        self.schedule
            .map(|schedule| notional.checked_mul_bps(schedule.protocol_fee_bps))
            .unwrap_or_else(|| Ok(Amount::zero()))
    }

    pub fn insurance_fee(&self, notional: Amount) -> FusionResult<Amount> {
        self.schedule
            .map(|schedule| notional.checked_mul_bps(schedule.insurance_fee_bps))
            .unwrap_or_else(|| Ok(Amount::zero()))
    }

    pub fn accrue(&mut self, asset: AssetId, amount: Amount) -> FusionResult<()> {
        if amount.is_zero() {
            return Ok(());
        }
        let current = self
            .accrued
            .get(&asset)
            .copied()
            .unwrap_or_else(Amount::zero);
        self.accrued.insert(asset, current.checked_add(amount)?);
        Ok(())
    }

    pub fn absorb_insurance_fee(&mut self, asset: AssetId, amount: Amount) -> FusionResult<()> {
        if amount.is_zero() {
            return Ok(());
        }
        let Some(mut reserve) = self.insurance else {
            return Ok(());
        };
        if reserve.asset == asset {
            reserve.balance = reserve.balance.checked_add(amount)?;
            self.insurance = Some(reserve);
        }
        Ok(())
    }

    pub fn accrued_asset_count(&self) -> usize {
        self.accrued.len()
    }
}
