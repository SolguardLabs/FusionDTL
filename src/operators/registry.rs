use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, Digest, FusionError, FusionResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorRole {
    Issuer,
    Beneficiary,
    Relayer,
    CellController,
    RiskAdmin,
    Treasury,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProtocolConfig {
    pub admin: AccountId,
    pub current_epoch: u64,
    pub review_delay_epochs: u64,
    pub config_digest: Digest,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct OperatorRegistry {
    roles: BTreeMap<AccountId, Vec<OperatorRole>>,
    config: Option<ProtocolConfig>,
    paused: bool,
}

impl ProtocolConfig {
    pub fn new(
        admin: AccountId,
        current_epoch: u64,
        review_delay_epochs: u64,
        salt: Digest,
    ) -> FusionResult<Self> {
        let config_digest = Digest::from_serializable(
            "fusion-protocol-config-v1",
            &(admin, current_epoch, review_delay_epochs, salt),
        )?;
        Ok(Self {
            admin,
            current_epoch,
            review_delay_epochs,
            config_digest,
        })
    }
}

impl OperatorRegistry {
    pub fn configure(&mut self, config: ProtocolConfig) {
        self.config = Some(config);
    }

    pub fn grant_role(&mut self, account: AccountId, role: OperatorRole) {
        let roles = self.roles.entry(account).or_default();
        if !roles.contains(&role) {
            roles.push(role);
            roles.sort();
        }
    }

    pub fn require_role(&self, account: AccountId, role: OperatorRole) -> FusionResult<()> {
        if self.has_role(account, role) {
            return Ok(());
        }
        Err(FusionError::Policy(
            "operator role is not assigned".to_owned(),
        ))
    }

    pub fn has_role(&self, account: AccountId, role: OperatorRole) -> bool {
        self.roles
            .get(&account)
            .is_some_and(|roles| roles.contains(&role))
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn ensure_not_paused(&self) -> FusionResult<()> {
        if self.paused {
            return Err(FusionError::Policy("protocol is paused".to_owned()));
        }
        Ok(())
    }

    pub fn operator_count(&self) -> usize {
        self.roles.len()
    }

    pub fn role_assignment_count(&self) -> usize {
        self.roles.values().map(Vec::len).sum()
    }
}
