use std::collections::BTreeSet;

use serde::Serialize;

use crate::{Amount, Bps, CellId, FusionError, FusionResult};

const BPS_DENOMINATOR: u128 = 10_000;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CellLiquidityInput {
    pub cell_id: CellId,
    pub reserve: Amount,
    pub pending_liability: Amount,
    pub forecast_outflow: Amount,
    pub forecast_inflow: Amount,
    pub reserve_haircut: Bps,
    pub confidence_bps: Bps,
    pub concentration_limit: Bps,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LiquidityControlPolicy {
    pub min_confidence_bps: Bps,
    pub inflow_recovery_bps: Bps,
    pub outflow_surge_bps: Bps,
    pub target_coverage_bps: u16,
    pub minimum_coverage_bps: u16,
    pub halt_coverage_bps: u16,
}

impl Default for LiquidityControlPolicy {
    fn default() -> Self {
        Self {
            min_confidence_bps: Bps::new(9_700).expect("valid basis points"),
            inflow_recovery_bps: Bps::new(7_000).expect("valid basis points"),
            outflow_surge_bps: Bps::new(2_000).expect("valid basis points"),
            target_coverage_bps: 12_500,
            minimum_coverage_bps: 10_000,
            halt_coverage_bps: 9_000,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LiquidityBand {
    Healthy,
    Watch,
    Restricted,
    Halted,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CellLiquidityAssessment {
    pub cell_id: CellId,
    pub stressed_reserve: Amount,
    pub recoverable_inflow: Amount,
    pub stressed_outflow: Amount,
    pub total_commitment: Amount,
    pub available_after_commitments: Amount,
    pub liquidity_gap: Amount,
    pub coverage_bps: u16,
    pub utilization_bps: u16,
    pub concentration_bps: u16,
    pub confidence_breach: bool,
    pub concentration_breach: bool,
    pub band: LiquidityBand,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LiquidityControlReport {
    pub band: LiquidityBand,
    pub cells: Vec<CellLiquidityAssessment>,
    pub stressed_resources: Amount,
    pub total_commitments: Amount,
    pub available_after_commitments: Amount,
    pub liquidity_gap: Amount,
    pub coverage_bps: u16,
    pub largest_concentration_bps: u16,
    pub confidence_breaches: usize,
    pub concentration_breaches: usize,
}

impl LiquidityControlReport {
    pub fn accepts_new_receipts(&self) -> bool {
        matches!(self.band, LiquidityBand::Healthy | LiquidityBand::Watch)
    }

    pub fn requires_reconciliation(&self) -> bool {
        matches!(self.band, LiquidityBand::Restricted | LiquidityBand::Halted)
    }
}

pub struct LiquidityControlEngine;

impl LiquidityControlEngine {
    pub fn assess(
        policy: LiquidityControlPolicy,
        inputs: &[CellLiquidityInput],
    ) -> FusionResult<LiquidityControlReport> {
        Self::validate_policy(policy)?;
        if inputs.is_empty() {
            return Err(FusionError::Policy(
                "liquidity control requires at least one cell".to_owned(),
            ));
        }

        let mut seen = BTreeSet::new();
        let mut provisional = Vec::with_capacity(inputs.len());
        let mut stressed_resources = 0u128;
        let mut total_commitments = 0u128;

        for input in inputs {
            if !seen.insert(input.cell_id) {
                return Err(FusionError::Policy(
                    "duplicate cell in liquidity control input".to_owned(),
                ));
            }

            let retained_bps = 10_000u16
                .checked_sub(input.reserve_haircut.units())
                .ok_or(FusionError::BpsOutOfRange(input.reserve_haircut.units()))?;
            let stressed_reserve = Self::mul_bps(input.reserve.units(), retained_bps)?;
            let recoverable_inflow = Self::mul_bps(
                input.forecast_inflow.units(),
                policy.inflow_recovery_bps.units(),
            )?;
            let surge = Self::mul_bps(
                input.forecast_outflow.units(),
                policy.outflow_surge_bps.units(),
            )?;
            let stressed_outflow = input
                .forecast_outflow
                .units()
                .checked_add(surge)
                .ok_or(FusionError::AmountOverflow)?;
            let total_commitment = input
                .pending_liability
                .units()
                .checked_add(stressed_outflow)
                .ok_or(FusionError::AmountOverflow)?;
            let resources = stressed_reserve
                .checked_add(recoverable_inflow)
                .ok_or(FusionError::AmountOverflow)?;

            stressed_resources = stressed_resources
                .checked_add(resources)
                .ok_or(FusionError::AmountOverflow)?;
            total_commitments = total_commitments
                .checked_add(total_commitment)
                .ok_or(FusionError::AmountOverflow)?;
            provisional.push((
                *input,
                stressed_reserve,
                recoverable_inflow,
                stressed_outflow,
                total_commitment,
                resources,
            ));
        }

        let mut assessments = Vec::with_capacity(provisional.len());
        let mut largest_concentration_bps = 0u16;
        let mut confidence_breaches = 0usize;
        let mut concentration_breaches = 0usize;

        for (
            input,
            stressed_reserve,
            recoverable_inflow,
            stressed_outflow,
            commitment,
            resources,
        ) in provisional
        {
            let coverage_bps = Self::ratio_bps(resources, commitment);
            let utilization_bps = Self::ratio_bps(commitment, resources);
            let concentration_bps = Self::ratio_bps(resources, stressed_resources);
            let confidence_breach = input.confidence_bps < policy.min_confidence_bps;
            let concentration_breach = concentration_bps > input.concentration_limit.units();
            confidence_breaches += usize::from(confidence_breach);
            concentration_breaches += usize::from(concentration_breach);
            largest_concentration_bps = largest_concentration_bps.max(concentration_bps);
            let band = Self::classify(
                policy,
                coverage_bps,
                confidence_breach,
                concentration_breach,
            );

            assessments.push(CellLiquidityAssessment {
                cell_id: input.cell_id,
                stressed_reserve: Amount::new(stressed_reserve)?,
                recoverable_inflow: Amount::new(recoverable_inflow)?,
                stressed_outflow: Amount::new(stressed_outflow)?,
                total_commitment: Amount::new(commitment)?,
                available_after_commitments: Amount::new(resources.saturating_sub(commitment))?,
                liquidity_gap: Amount::new(commitment.saturating_sub(resources))?,
                coverage_bps,
                utilization_bps,
                concentration_bps,
                confidence_breach,
                concentration_breach,
                band,
            });
        }

        let coverage_bps = Self::ratio_bps(stressed_resources, total_commitments);
        let aggregate_band = assessments
            .iter()
            .map(|assessment| assessment.band)
            .max()
            .unwrap_or(LiquidityBand::Halted);

        Ok(LiquidityControlReport {
            band: aggregate_band,
            cells: assessments,
            stressed_resources: Amount::new(stressed_resources)?,
            total_commitments: Amount::new(total_commitments)?,
            available_after_commitments: Amount::new(
                stressed_resources.saturating_sub(total_commitments),
            )?,
            liquidity_gap: Amount::new(total_commitments.saturating_sub(stressed_resources))?,
            coverage_bps,
            largest_concentration_bps,
            confidence_breaches,
            concentration_breaches,
        })
    }

    fn validate_policy(policy: LiquidityControlPolicy) -> FusionResult<()> {
        if policy.halt_coverage_bps > policy.minimum_coverage_bps
            || policy.minimum_coverage_bps > policy.target_coverage_bps
        {
            return Err(FusionError::Policy(
                "liquidity coverage thresholds are not monotonic".to_owned(),
            ));
        }
        Ok(())
    }

    fn mul_bps(value: u128, bps: u16) -> FusionResult<u128> {
        value
            .checked_mul(u128::from(bps))
            .and_then(|scaled| scaled.checked_div(BPS_DENOMINATOR))
            .ok_or(FusionError::AmountOverflow)
    }

    fn ratio_bps(numerator: u128, denominator: u128) -> u16 {
        if denominator == 0 {
            return if numerator == 0 { 10_000 } else { u16::MAX };
        }
        numerator
            .saturating_mul(BPS_DENOMINATOR)
            .checked_div(denominator)
            .unwrap_or(u128::from(u16::MAX))
            .min(u128::from(u16::MAX)) as u16
    }

    fn classify(
        policy: LiquidityControlPolicy,
        coverage_bps: u16,
        confidence_breach: bool,
        concentration_breach: bool,
    ) -> LiquidityBand {
        if coverage_bps < policy.halt_coverage_bps {
            LiquidityBand::Halted
        } else if coverage_bps < policy.minimum_coverage_bps || confidence_breach {
            LiquidityBand::Restricted
        } else if coverage_bps < policy.target_coverage_bps || concentration_breach {
            LiquidityBand::Watch
        } else {
            LiquidityBand::Healthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(byte: u8) -> CellId {
        CellId::from_bytes([byte; 32])
    }

    fn input(cell: u8, reserve: u128, pending: u128) -> CellLiquidityInput {
        CellLiquidityInput {
            cell_id: id(cell),
            reserve: Amount::new(reserve).unwrap(),
            pending_liability: Amount::new(pending).unwrap(),
            forecast_outflow: Amount::new(0).unwrap(),
            forecast_inflow: Amount::new(0).unwrap(),
            reserve_haircut: Bps::new(500).unwrap(),
            confidence_bps: Bps::new(9_900).unwrap(),
            concentration_limit: Bps::new(7_000).unwrap(),
        }
    }

    #[test]
    fn balanced_cells_are_healthy() {
        let report = LiquidityControlEngine::assess(
            LiquidityControlPolicy::default(),
            &[input(1, 200_000, 100_000), input(2, 200_000, 100_000)],
        )
        .unwrap();
        assert_eq!(report.band, LiquidityBand::Healthy);
        assert!(report.accepts_new_receipts());
        assert!(!report.requires_reconciliation());
    }

    #[test]
    fn reserve_haircut_is_applied_before_coverage() {
        let report = LiquidityControlEngine::assess(
            LiquidityControlPolicy::default(),
            &[input(1, 200_000, 100_000)],
        )
        .unwrap();
        assert_eq!(report.cells[0].stressed_reserve.units(), 190_000);
        assert_eq!(report.cells[0].coverage_bps, 19_000);
    }

    #[test]
    fn forecast_outflow_receives_surge_factor() {
        let mut cell = input(1, 200_000, 100_000);
        cell.forecast_outflow = Amount::new(50_000).unwrap();
        let report =
            LiquidityControlEngine::assess(LiquidityControlPolicy::default(), &[cell]).unwrap();
        assert_eq!(report.cells[0].stressed_outflow.units(), 60_000);
        assert_eq!(report.cells[0].total_commitment.units(), 160_000);
    }

    #[test]
    fn forecast_inflow_is_recovered_conservatively() {
        let mut cell = input(1, 100_000, 100_000);
        cell.forecast_inflow = Amount::new(50_000).unwrap();
        let report =
            LiquidityControlEngine::assess(LiquidityControlPolicy::default(), &[cell]).unwrap();
        assert_eq!(report.cells[0].recoverable_inflow.units(), 35_000);
        assert_eq!(report.cells[0].coverage_bps, 13_000);
    }

    #[test]
    fn low_confidence_restricts_admission() {
        let mut cell = input(1, 300_000, 100_000);
        cell.confidence_bps = Bps::new(9_500).unwrap();
        let report =
            LiquidityControlEngine::assess(LiquidityControlPolicy::default(), &[cell]).unwrap();
        assert_eq!(report.band, LiquidityBand::Restricted);
        assert_eq!(report.confidence_breaches, 1);
    }

    #[test]
    fn concentration_moves_portfolio_to_watch() {
        let mut large = input(1, 900_000, 100_000);
        large.concentration_limit = Bps::new(7_000).unwrap();
        let report = LiquidityControlEngine::assess(
            LiquidityControlPolicy::default(),
            &[large, input(2, 100_000, 20_000)],
        )
        .unwrap();
        assert_eq!(report.band, LiquidityBand::Watch);
        assert_eq!(report.largest_concentration_bps, 9_000);
        assert_eq!(report.concentration_breaches, 1);
    }

    #[test]
    fn sub_target_coverage_moves_cell_to_watch() {
        let report = LiquidityControlEngine::assess(
            LiquidityControlPolicy::default(),
            &[input(1, 125_000, 100_000)],
        )
        .unwrap();
        assert_eq!(report.band, LiquidityBand::Watch);
    }

    #[test]
    fn material_gap_halts_cell() {
        let report = LiquidityControlEngine::assess(
            LiquidityControlPolicy::default(),
            &[input(1, 80_000, 100_000)],
        )
        .unwrap();
        assert_eq!(report.band, LiquidityBand::Halted);
        assert_eq!(report.liquidity_gap.units(), 24_000);
        assert!(report.requires_reconciliation());
    }

    #[test]
    fn duplicate_cells_are_rejected() {
        let result = LiquidityControlEngine::assess(
            LiquidityControlPolicy::default(),
            &[input(1, 100, 10), input(1, 100, 10)],
        );
        assert!(
            matches!(result, Err(FusionError::Policy(message)) if message.contains("duplicate"))
        );
    }

    #[test]
    fn empty_input_is_rejected() {
        let result = LiquidityControlEngine::assess(LiquidityControlPolicy::default(), &[]);
        assert!(
            matches!(result, Err(FusionError::Policy(message)) if message.contains("at least one"))
        );
    }

    #[test]
    fn non_monotonic_thresholds_are_rejected() {
        let policy = LiquidityControlPolicy {
            minimum_coverage_bps: 8_000,
            halt_coverage_bps: 9_000,
            ..LiquidityControlPolicy::default()
        };
        let result = LiquidityControlEngine::assess(policy, &[input(1, 100, 10)]);
        assert!(
            matches!(result, Err(FusionError::Policy(message)) if message.contains("monotonic"))
        );
    }

    #[test]
    fn arithmetic_overflow_fails_closed() {
        let mut cell = input(1, u128::MAX, 1);
        cell.forecast_inflow = Amount::new(u128::MAX).unwrap();
        let result = LiquidityControlEngine::assess(LiquidityControlPolicy::default(), &[cell]);
        assert_eq!(result, Err(FusionError::AmountOverflow));
    }
}
