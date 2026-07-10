use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, Amount, AssetId, FusionError, FusionResult, PublicIdentity};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AccountState {
    pub identity: PublicIdentity,
    pub balances: BTreeMap<AssetId, Amount>,
    pub next_receipt_nonce: u64,
    pub next_packet_nonce: u64,
}

impl AccountState {
    pub fn new(identity: PublicIdentity) -> Self {
        Self {
            identity,
            balances: BTreeMap::new(),
            next_receipt_nonce: 0,
            next_packet_nonce: 0,
        }
    }

    pub fn balance_of(&self, asset: AssetId) -> Amount {
        self.balances
            .get(&asset)
            .copied()
            .unwrap_or_else(Amount::zero)
    }

    pub(crate) fn credit(&mut self, asset: AssetId, amount: Amount) -> FusionResult<()> {
        let next = self.balance_of(asset).checked_add(amount)?;
        self.set_balance(asset, next);
        Ok(())
    }

    pub(crate) fn debit(
        &mut self,
        account: AccountId,
        asset: AssetId,
        amount: Amount,
    ) -> FusionResult<()> {
        let available = self.balance_of(asset);
        if available < amount {
            return Err(FusionError::InsufficientFunds {
                account,
                asset,
                available,
                required: amount,
            });
        }
        self.set_balance(asset, available.checked_sub(amount)?);
        Ok(())
    }

    pub(crate) fn advance_receipt_nonce(&mut self) -> FusionResult<()> {
        self.next_receipt_nonce = self
            .next_receipt_nonce
            .checked_add(1)
            .ok_or(FusionError::AmountOverflow)?;
        Ok(())
    }

    pub(crate) fn advance_packet_nonce(&mut self) -> FusionResult<()> {
        self.next_packet_nonce = self
            .next_packet_nonce
            .checked_add(1)
            .ok_or(FusionError::AmountOverflow)?;
        Ok(())
    }

    fn set_balance(&mut self, asset: AssetId, amount: Amount) {
        if amount.is_zero() {
            self.balances.remove(&asset);
        } else {
            self.balances.insert(asset, amount);
        }
    }
}
