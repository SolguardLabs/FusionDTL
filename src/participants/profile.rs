use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, Digest, FusionError, FusionResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantTier {
    Retail,
    Professional,
    Institution,
    Protocol,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantStatus {
    Active,
    Review,
    Suspended,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    Global,
    Eu,
    Uk,
    Us,
    Apac,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ParticipantProfile {
    pub account: AccountId,
    pub tier: ParticipantTier,
    pub jurisdiction: Jurisdiction,
    pub status: ParticipantStatus,
    pub expires_at_epoch: u64,
    pub attestation_digest: Digest,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ScreeningBook {
    profiles: BTreeMap<AccountId, ParticipantProfile>,
}

impl ParticipantProfile {
    pub fn new(
        account: AccountId,
        tier: ParticipantTier,
        jurisdiction: Jurisdiction,
        status: ParticipantStatus,
        expires_at_epoch: u64,
        salt: Digest,
    ) -> FusionResult<Self> {
        if expires_at_epoch == 0 {
            return Err(FusionError::Policy(
                "participant profile expiry is invalid".to_owned(),
            ));
        }
        let attestation_digest = Digest::from_serializable(
            "fusion-participant-profile-v1",
            &(account, tier, jurisdiction, status, expires_at_epoch, salt),
        )?;
        Ok(Self {
            account,
            tier,
            jurisdiction,
            status,
            expires_at_epoch,
            attestation_digest,
        })
    }
}

impl ScreeningBook {
    pub fn attest(&mut self, profile: ParticipantProfile) -> Digest {
        self.profiles.insert(profile.account, profile);
        profile.attestation_digest
    }

    pub fn require_active(&self, account: AccountId, epoch: u64) -> FusionResult<()> {
        let profile = self
            .profiles
            .get(&account)
            .ok_or_else(|| FusionError::Policy("participant profile missing".to_owned()))?;
        if profile.status != ParticipantStatus::Active {
            return Err(FusionError::Policy(
                "participant profile is not active".to_owned(),
            ));
        }
        if epoch > profile.expires_at_epoch {
            return Err(FusionError::Policy(
                "participant profile has expired".to_owned(),
            ));
        }
        Ok(())
    }

    pub fn profile_count(&self) -> usize {
        self.profiles.len()
    }

    pub fn active_profile_count(&self) -> usize {
        self.profiles
            .values()
            .filter(|profile| profile.status == ParticipantStatus::Active)
            .count()
    }
}
