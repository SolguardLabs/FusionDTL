use std::collections::BTreeMap;

use serde::Serialize;

use crate::{Amount, AssetId, CellId, Digest, FusionError, FusionResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CellCapacityPolicy {
    pub policy_id: Digest,
    pub cell_id: CellId,
    pub asset: AssetId,
    pub soft_cap: Amount,
    pub hard_cap: Amount,
    pub max_single_issue: Amount,
    pub review_epoch: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct CapacityBook {
    policies: BTreeMap<CellId, CellCapacityPolicy>,
}

impl CellCapacityPolicy {
    pub fn new(
        cell_id: CellId,
        asset: AssetId,
        soft_cap: Amount,
        hard_cap: Amount,
        max_single_issue: Amount,
        review_epoch: u64,
        salt: Digest,
    ) -> FusionResult<Self> {
        if soft_cap.is_zero() || hard_cap.is_zero() || max_single_issue.is_zero() {
            return Err(FusionError::ZeroAmount);
        }
        if soft_cap > hard_cap {
            return Err(FusionError::Policy(
                "capacity soft cap exceeds hard cap".to_owned(),
            ));
        }
        if max_single_issue > hard_cap {
            return Err(FusionError::Policy(
                "single issue limit exceeds hard cap".to_owned(),
            ));
        }
        let policy_id = Digest::from_serializable(
            "fusion-cell-capacity-policy-v1",
            &(
                cell_id,
                asset,
                soft_cap,
                hard_cap,
                max_single_issue,
                review_epoch,
                salt,
            ),
        )?;
        Ok(Self {
            policy_id,
            cell_id,
            asset,
            soft_cap,
            hard_cap,
            max_single_issue,
            review_epoch,
        })
    }
}

impl CapacityBook {
    pub fn configure(&mut self, policy: CellCapacityPolicy) -> Digest {
        self.policies.insert(policy.cell_id, policy);
        policy.policy_id
    }

    pub fn ensure_issue_allowed(
        &self,
        cell_id: CellId,
        asset: AssetId,
        amount: Amount,
        pending_liability: Amount,
    ) -> FusionResult<()> {
        let Some(policy) = self.policies.get(&cell_id) else {
            return Ok(());
        };
        if policy.asset != asset {
            return Err(FusionError::Policy("capacity asset mismatch".to_owned()));
        }
        if amount > policy.max_single_issue {
            return Err(FusionError::Policy(
                "single issue amount exceeds capacity policy".to_owned(),
            ));
        }
        let next_pending = pending_liability.checked_add(amount)?;
        if next_pending > policy.hard_cap {
            return Err(FusionError::Policy(
                "cell capacity hard cap reached".to_owned(),
            ));
        }
        Ok(())
    }

    pub fn policy_count(&self) -> usize {
        self.policies.len()
    }
}
