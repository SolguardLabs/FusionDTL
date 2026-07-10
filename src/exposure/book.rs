use std::collections::BTreeMap;

use serde::Serialize;

use crate::{Amount, CellId, FusionResult};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct CellExposure {
    pub issued: Amount,
    pub settled: Amount,
    pub routed_in: Amount,
    pub routed_out: Amount,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ExposureBook {
    cells: BTreeMap<CellId, CellExposure>,
}

impl ExposureBook {
    pub fn record_issue(&mut self, cell_id: CellId, amount: Amount) -> FusionResult<()> {
        let mut exposure = self.exposure_of(cell_id);
        exposure.issued = exposure.issued.checked_add(amount)?;
        self.cells.insert(cell_id, exposure);
        Ok(())
    }

    pub fn record_settlement(
        &mut self,
        source_cell: CellId,
        payout_cell: CellId,
        amount: Amount,
    ) -> FusionResult<()> {
        let mut source = self.exposure_of(source_cell);
        source.settled = source.settled.checked_add(amount)?;
        source.routed_out = source.routed_out.checked_add(amount)?;
        self.cells.insert(source_cell, source);

        let mut payout = self.exposure_of(payout_cell);
        payout.routed_in = payout.routed_in.checked_add(amount)?;
        self.cells.insert(payout_cell, payout);
        Ok(())
    }

    pub fn exposure_of(&self, cell_id: CellId) -> CellExposure {
        self.cells.get(&cell_id).copied().unwrap_or_default()
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }
}
