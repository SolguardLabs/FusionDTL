use std::collections::BTreeMap;

use serde::Serialize;

use crate::{Digest, FusionError, FusionResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SettlementWindow {
    pub window_id: Digest,
    pub opens_at_epoch: u64,
    pub closes_at_epoch: u64,
    pub min_confirmations: u16,
    pub soft_cutoff_epoch: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct SettlementCalendar {
    windows: BTreeMap<Digest, SettlementWindow>,
}

impl SettlementWindow {
    pub fn new(
        opens_at_epoch: u64,
        closes_at_epoch: u64,
        min_confirmations: u16,
        soft_cutoff_epoch: u64,
        salt: Digest,
    ) -> FusionResult<Self> {
        if opens_at_epoch >= closes_at_epoch {
            return Err(FusionError::Policy(
                "settlement window range is invalid".to_owned(),
            ));
        }
        if soft_cutoff_epoch < opens_at_epoch || soft_cutoff_epoch > closes_at_epoch {
            return Err(FusionError::Policy(
                "settlement soft cutoff is outside the window".to_owned(),
            ));
        }
        let window_id = Digest::from_serializable(
            "fusion-settlement-window-v1",
            &(
                opens_at_epoch,
                closes_at_epoch,
                min_confirmations,
                soft_cutoff_epoch,
                salt,
            ),
        )?;
        Ok(Self {
            window_id,
            opens_at_epoch,
            closes_at_epoch,
            min_confirmations,
            soft_cutoff_epoch,
        })
    }

    pub const fn accepts(self, epoch: u64) -> bool {
        epoch >= self.opens_at_epoch && epoch <= self.closes_at_epoch
    }
}

impl SettlementCalendar {
    pub fn register_window(&mut self, window: SettlementWindow) -> FusionResult<Digest> {
        if self.windows.contains_key(&window.window_id) {
            return Err(FusionError::Policy(
                "settlement window already exists".to_owned(),
            ));
        }
        self.windows.insert(window.window_id, window);
        Ok(window.window_id)
    }

    pub fn ensure_open(&self, epoch: u64) -> FusionResult<SettlementWindow> {
        self.windows
            .values()
            .copied()
            .find(|window| window.accepts(epoch))
            .ok_or_else(|| FusionError::Policy("settlement window is closed".to_owned()))
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }
}
