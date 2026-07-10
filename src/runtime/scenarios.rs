use serde::Serialize;

use crate::{
    Amount, AssetConfig, AssetId, Bps, CellCapacityPolicy, CellConfig, CellId, DeliveryLane,
    DeliveryReceipt, Digest, FeeSchedule, FusionLedger, FusionResult, InsuranceReserve,
    Jurisdiction, KeyPair, OperatorRole, ParticipantProfile, ParticipantStatus, ParticipantTier,
    PriceObservation, ProtocolConfig, ReceiptId, ReceiptOrder, RelayerQuote, RiskLimits,
    SettlementPacket, SettlementWindow, SignedReceiptOrder, SignedSettlementPacket, TxId,
    VenueConfig,
};

const NETWORK_ID: u32 = 61_804;

struct Fixture {
    ledger: FusionLedger,
    issuer: KeyPair,
    beneficiary: KeyPair,
    relayer: KeyPair,
    core_lp: KeyPair,
    core_controller: KeyPair,
    edge_controller: KeyPair,
    asset: AssetConfig,
    core_cell: CellId,
    edge_cell: CellId,
}

#[derive(Debug, Serialize)]
pub struct ScenarioReport {
    pub scenario: String,
    pub network_id: u32,
    pub asset: AssetReport,
    pub balances: BalanceReport,
    pub cells: CellReport,
    pub receipt_id: Option<ReceiptId>,
    pub receipt: Option<DeliveryReceipt>,
    pub transactions: Vec<TxId>,
    pub surface: SurfaceReport,
    pub journal_entries: usize,
    pub state_digest: Digest,
    pub conservation_ok: bool,
}

#[derive(Debug, Serialize)]
pub struct AssetReport {
    pub id: AssetId,
    pub symbol: &'static str,
    pub decimals: u8,
}

#[derive(Debug, Serialize)]
pub struct BalanceReport {
    pub issuer: Amount,
    pub beneficiary: Amount,
    pub relayer: Amount,
    pub core_lp: Amount,
    pub core_controller: Amount,
    pub edge_controller: Amount,
}

#[derive(Debug, Serialize)]
pub struct CellReport {
    pub core_reserve: Amount,
    pub core_pending: Amount,
    pub edge_reserve: Amount,
    pub edge_pending: Amount,
}

#[derive(Debug, Serialize)]
pub struct SurfaceReport {
    pub receipts: usize,
    pub processed_packets: usize,
    pub oracle_markets: usize,
    pub participant_profiles: usize,
    pub active_profiles: usize,
    pub settlement_windows: usize,
    pub operators: usize,
    pub role_assignments: usize,
    pub delivery_lanes: usize,
    pub relayer_quotes: usize,
    pub treasury_assets: usize,
    pub exposure_cells: usize,
    pub capacity_policies: usize,
}

pub fn run_named(name: &str) -> FusionResult<ScenarioReport> {
    match name {
        "snapshot" => snapshot(),
        "issue" => issue(),
        "settle" => settle(),
        "rebalance" => rebalance(),
        _ => settle(),
    }
}

fn snapshot() -> FusionResult<ScenarioReport> {
    report(fixture()?, "snapshot", None, Vec::new())
}

fn issue() -> FusionResult<ScenarioReport> {
    let mut fixture = fixture()?;
    let (receipt, open_tx) = issue_delivery_receipt(&mut fixture, 2_500_000_000, "issue")?;
    report(fixture, "issue", Some(receipt), vec![open_tx])
}

fn settle() -> FusionResult<ScenarioReport> {
    let mut fixture = fixture()?;
    let (receipt, open_tx) = issue_delivery_receipt(&mut fixture, 2_500_000_000, "settle")?;
    let payout_cell = fixture.edge_cell;
    let settle_tx = settle_delivery_receipt(&mut fixture, receipt, payout_cell)?;
    report(fixture, "settle", Some(receipt), vec![open_tx, settle_tx])
}

fn rebalance() -> FusionResult<ScenarioReport> {
    let mut fixture = fixture()?;
    let (receipt, open_tx) = issue_delivery_receipt(&mut fixture, 2_500_000_000, "rebalance")?;
    let payout_cell = fixture.core_cell;
    let settle_tx = settle_delivery_receipt(&mut fixture, receipt, payout_cell)?;
    let sweep_tx = fixture.ledger.sweep_cell_surplus(
        fixture.edge_controller.public_identity().account,
        fixture.edge_cell,
        Amount::new(2_500_000_000)?,
    )?;
    report(
        fixture,
        "rebalance",
        Some(receipt),
        vec![open_tx, settle_tx, sweep_tx],
    )
}

fn fixture() -> FusionResult<Fixture> {
    let issuer = keyed(11);
    let beneficiary = keyed(22);
    let relayer = keyed(33);
    let core_lp = keyed(44);
    let core_controller = keyed(55);
    let edge_controller = keyed(66);
    let asset = AssetConfig::new("FUSD", 6)?;
    let venue = VenueConfig::new("fusion-main", 2, Bps::new(60)?);
    let mut ledger = FusionLedger::new(NETWORK_ID, venue);
    ledger.set_risk_limits(RiskLimits::new(
        900,
        Amount::new(20_000_000_000)?,
        Bps::new(60)?,
        Amount::zero(),
    )?);
    ledger.register_asset(asset)?;
    for identity in [
        issuer.public_identity(),
        beneficiary.public_identity(),
        relayer.public_identity(),
        core_lp.public_identity(),
        core_controller.public_identity(),
        edge_controller.public_identity(),
    ] {
        ledger.register_account(identity)?;
    }
    for (account, tier, jurisdiction, salt) in [
        (
            issuer.public_identity().account,
            ParticipantTier::Institution,
            Jurisdiction::Eu,
            b"issuer" as &[u8],
        ),
        (
            beneficiary.public_identity().account,
            ParticipantTier::Professional,
            Jurisdiction::Eu,
            b"beneficiary" as &[u8],
        ),
        (
            relayer.public_identity().account,
            ParticipantTier::Protocol,
            Jurisdiction::Global,
            b"relayer" as &[u8],
        ),
        (
            core_lp.public_identity().account,
            ParticipantTier::Institution,
            Jurisdiction::Uk,
            b"core-lp" as &[u8],
        ),
        (
            core_controller.public_identity().account,
            ParticipantTier::Protocol,
            Jurisdiction::Global,
            b"core-controller" as &[u8],
        ),
        (
            edge_controller.public_identity().account,
            ParticipantTier::Protocol,
            Jurisdiction::Global,
            b"edge-controller" as &[u8],
        ),
    ] {
        ledger.attest_participant(ParticipantProfile::new(
            account,
            tier,
            jurisdiction,
            ParticipantStatus::Active,
            1_200,
            Digest::from_parts("fusion-profile-salt-v1", &[salt]),
        )?)?;
    }
    ledger.configure_protocol(ProtocolConfig::new(
        core_controller.public_identity().account,
        900,
        2,
        Digest::from_parts("fusion-protocol-config-salt-v1", &[b"primary"]),
    )?)?;
    ledger.register_settlement_window(SettlementWindow::new(
        875,
        950,
        2,
        930,
        Digest::from_parts("fusion-window-salt-v1", &[b"primary"]),
    )?)?;
    for (account, role) in [
        (issuer.public_identity().account, OperatorRole::Issuer),
        (
            beneficiary.public_identity().account,
            OperatorRole::Beneficiary,
        ),
        (relayer.public_identity().account, OperatorRole::Relayer),
        (
            core_controller.public_identity().account,
            OperatorRole::CellController,
        ),
        (
            edge_controller.public_identity().account,
            OperatorRole::CellController,
        ),
        (
            core_controller.public_identity().account,
            OperatorRole::RiskAdmin,
        ),
        (
            core_controller.public_identity().account,
            OperatorRole::Treasury,
        ),
    ] {
        ledger.grant_operator_role(account, role)?;
    }
    ledger.credit_genesis(
        issuer.public_identity().account,
        asset.id,
        Amount::new(20_000_000_000)?,
    )?;
    ledger.credit_genesis(
        core_lp.public_identity().account,
        asset.id,
        Amount::new(120_000_000_000)?,
    )?;
    ledger.publish_price(PriceObservation::new(
        asset.id,
        asset.id,
        1,
        1,
        Bps::new(5)?,
        900,
    )?);
    let core_config = CellConfig::new(
        core_controller.public_identity().account,
        asset.id,
        1,
        Amount::new(5_000_000_000)?,
        Bps::new(8_000)?,
        Digest::from_parts("fusion-core-cell-salt-v1", &[b"core"]),
    );
    let edge_config = CellConfig::new(
        edge_controller.public_identity().account,
        asset.id,
        2,
        Amount::zero(),
        Bps::new(10_000)?,
        Digest::from_parts("fusion-edge-cell-salt-v1", &[b"edge"]),
    );
    let core_cell = core_config.cell_id;
    let edge_cell = edge_config.cell_id;
    ledger.register_cell(core_config)?;
    ledger.register_cell(edge_config)?;
    ledger.configure_cell_capacity(CellCapacityPolicy::new(
        edge_cell,
        asset.id,
        Amount::new(12_500_000_000)?,
        Amount::new(20_000_000_000)?,
        Amount::new(5_000_000_000)?,
        940,
        Digest::from_parts("fusion-capacity-salt-v1", &[b"edge"]),
    )?)?;
    ledger.configure_cell_capacity(CellCapacityPolicy::new(
        core_cell,
        asset.id,
        Amount::new(80_000_000_000)?,
        Amount::new(120_000_000_000)?,
        Amount::new(25_000_000_000)?,
        940,
        Digest::from_parts("fusion-capacity-salt-v1", &[b"core"]),
    )?)?;
    ledger.configure_fee_schedule(FeeSchedule::new(
        core_controller.public_identity().account,
        Bps::new(500)?,
        Bps::new(250)?,
    )?)?;
    ledger.configure_insurance_reserve(InsuranceReserve::new(
        asset.id,
        Amount::new(1_000_000_000)?,
        Amount::new(500_000_000)?,
    )?)?;
    let edge_local_lane = DeliveryLane::new(
        edge_cell,
        edge_cell,
        asset.id,
        Amount::new(100_000_000)?,
        Amount::new(10_000_000_000)?,
        Bps::new(60)?,
        2,
        Digest::from_parts("fusion-lane-salt-v1", &[b"edge-local"]),
    )?;
    let edge_local_lane_id = edge_local_lane.lane_id;
    ledger.register_delivery_lane(edge_local_lane)?;
    let edge_core_lane = DeliveryLane::new(
        edge_cell,
        core_cell,
        asset.id,
        Amount::new(100_000_000)?,
        Amount::new(10_000_000_000)?,
        Bps::new(60)?,
        2,
        Digest::from_parts("fusion-lane-salt-v1", &[b"edge-core"]),
    )?;
    let edge_core_lane_id = edge_core_lane.lane_id;
    ledger.register_delivery_lane(edge_core_lane)?;
    ledger.submit_relayer_quote(RelayerQuote::new(
        edge_local_lane_id,
        relayer.public_identity().account,
        Amount::new(5_000_000)?,
        1,
        950,
    )?)?;
    ledger.submit_relayer_quote(RelayerQuote::new(
        edge_core_lane_id,
        relayer.public_identity().account,
        Amount::new(5_000_000)?,
        2,
        950,
    )?)?;
    ledger.deposit_cell(
        core_lp.public_identity().account,
        core_cell,
        Amount::new(90_000_000_000)?,
    )?;

    Ok(Fixture {
        ledger,
        issuer,
        beneficiary,
        relayer,
        core_lp,
        core_controller,
        edge_controller,
        asset,
        core_cell,
        edge_cell,
    })
}

fn issue_delivery_receipt(
    fixture: &mut Fixture,
    amount: u128,
    label: &'static str,
) -> FusionResult<(DeliveryReceipt, TxId)> {
    let issuer = fixture.issuer.public_identity().account;
    let beneficiary = fixture.beneficiary.public_identity().account;
    let nonce = fixture.ledger.receipt_nonce(issuer)?;
    let order = ReceiptOrder::new(
        fixture.ledger.network_id(),
        fixture.edge_cell,
        issuer,
        beneficiary,
        fixture.asset.id,
        Amount::new(amount)?,
        nonce,
        900,
        Digest::from_parts("fusion-route-v1", &[label.as_bytes()]),
    )?;
    let receipt = order.receipt();
    let signed = SignedReceiptOrder::sign(order, &fixture.issuer)?;
    let tx = fixture.ledger.issue_receipt(&signed)?;
    Ok((receipt, tx))
}

fn settle_delivery_receipt(
    fixture: &mut Fixture,
    receipt: DeliveryReceipt,
    payout_cell: CellId,
) -> FusionResult<TxId> {
    let beneficiary = fixture.beneficiary.public_identity().account;
    let relayer = fixture.relayer.public_identity().account;
    let packet = SettlementPacket::new(
        fixture.ledger.network_id(),
        payout_cell,
        receipt.receipt_id,
        beneficiary,
        relayer,
        Amount::new(5_000_000)?,
        fixture.ledger.packet_nonce(beneficiary)?,
        900,
        receipt.digest()?,
    )?;
    let signed = SignedSettlementPacket::sign(packet, &fixture.beneficiary)?;
    fixture.ledger.settle_packet(&signed)
}

fn report(
    fixture: Fixture,
    scenario: &'static str,
    receipt: Option<DeliveryReceipt>,
    transactions: Vec<TxId>,
) -> FusionResult<ScenarioReport> {
    let issuer = fixture.issuer.public_identity().account;
    let beneficiary = fixture.beneficiary.public_identity().account;
    let relayer = fixture.relayer.public_identity().account;
    let core_lp = fixture.core_lp.public_identity().account;
    let core_controller = fixture.core_controller.public_identity().account;
    let edge_controller = fixture.edge_controller.public_identity().account;
    let asset = fixture.asset.id;
    let core_cell = fixture.ledger.cell(fixture.core_cell)?;
    let edge_cell = fixture.ledger.cell(fixture.edge_cell)?;
    Ok(ScenarioReport {
        scenario: scenario.to_owned(),
        network_id: fixture.ledger.network_id(),
        asset: AssetReport {
            id: fixture.asset.id,
            symbol: fixture.asset.symbol,
            decimals: fixture.asset.decimals,
        },
        balances: BalanceReport {
            issuer: fixture.ledger.balance_of(issuer, asset)?,
            beneficiary: fixture.ledger.balance_of(beneficiary, asset)?,
            relayer: fixture.ledger.balance_of(relayer, asset)?,
            core_lp: fixture.ledger.balance_of(core_lp, asset)?,
            core_controller: fixture.ledger.balance_of(core_controller, asset)?,
            edge_controller: fixture.ledger.balance_of(edge_controller, asset)?,
        },
        cells: CellReport {
            core_reserve: core_cell.reserve_balance,
            core_pending: core_cell.pending_liability,
            edge_reserve: edge_cell.reserve_balance,
            edge_pending: edge_cell.pending_liability,
        },
        receipt_id: receipt.map(|value| value.receipt_id),
        receipt,
        transactions,
        surface: SurfaceReport {
            receipts: fixture.ledger.receipt_count(),
            processed_packets: fixture.ledger.processed_packet_count(),
            oracle_markets: fixture.ledger.oracle_market_count(),
            participant_profiles: fixture.ledger.participant_profile_count(),
            active_profiles: fixture.ledger.active_profile_count(),
            settlement_windows: fixture.ledger.settlement_window_count(),
            operators: fixture.ledger.operator_count(),
            role_assignments: fixture.ledger.role_assignment_count(),
            delivery_lanes: fixture.ledger.delivery_lane_count(),
            relayer_quotes: fixture.ledger.relayer_quote_count(),
            treasury_assets: fixture.ledger.treasury_asset_count(),
            exposure_cells: fixture.ledger.exposure_cell_count(),
            capacity_policies: fixture.ledger.capacity_policy_count(),
        },
        journal_entries: fixture.ledger.journal().len(),
        state_digest: fixture.ledger.state_digest()?,
        conservation_ok: fixture.ledger.is_conserved(asset)?,
    })
}

fn keyed(byte: u8) -> KeyPair {
    KeyPair::from_seed([byte; 32])
}
