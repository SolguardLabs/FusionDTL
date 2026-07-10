use serde::Serialize;

use crate::{
    AccountId, Amount, AssetId, Bps, CellId, Digest, Jurisdiction, OperatorRole, PacketId,
    ParticipantStatus, ParticipantTier, ProtocolConfig, ReceiptId, RiskSnapshot, TxId,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct JournalEntry {
    pub sequence: u64,
    pub tx_id: TxId,
    pub op: JournalOp,
    pub state_digest: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JournalOp {
    GenesisCredit {
        account: AccountId,
        asset: AssetId,
        amount: Amount,
    },
    CellRegistered {
        cell_id: CellId,
        controller: AccountId,
        reserve_asset: AssetId,
    },
    CellDeposit {
        cell_id: CellId,
        owner: AccountId,
        amount: Amount,
    },
    ProtocolConfigured {
        config: ProtocolConfig,
    },
    OperatorRoleGranted {
        account: AccountId,
        role: OperatorRole,
    },
    ParticipantAttested {
        account: AccountId,
        tier: ParticipantTier,
        jurisdiction: Jurisdiction,
        status: ParticipantStatus,
        expires_at_epoch: u64,
    },
    SettlementWindowRegistered {
        window_id: Digest,
        opens_at_epoch: u64,
        closes_at_epoch: u64,
    },
    DeliveryLaneRegistered {
        lane_id: Digest,
        source_cell: CellId,
        payout_cell: CellId,
        asset: AssetId,
    },
    RelayerQuoteSubmitted {
        lane_id: Digest,
        relayer: AccountId,
        quote_digest: Digest,
        relayer_fee: Amount,
    },
    FeeScheduleConfigured {
        treasury: AccountId,
        protocol_fee_bps: Bps,
        insurance_fee_bps: Bps,
    },
    InsuranceReserveConfigured {
        asset: AssetId,
        balance: Amount,
        floor: Amount,
    },
    CellCapacityConfigured {
        policy_id: Digest,
        cell_id: CellId,
        soft_cap: Amount,
        hard_cap: Amount,
    },
    ReceiptIssued {
        receipt_id: ReceiptId,
        source_cell: CellId,
        issuer: AccountId,
        beneficiary: AccountId,
        amount: Amount,
    },
    PacketSettled {
        packet_id: PacketId,
        receipt_id: ReceiptId,
        lane_id: Digest,
        payout_cell: CellId,
        source_cell: CellId,
        beneficiary: AccountId,
        beneficiary_amount: Amount,
        relayer: AccountId,
        relayer_fee: Amount,
        protocol_fee: Amount,
        insurance_fee: Amount,
        risk: Box<RiskSnapshot>,
    },
    CellSurplusSwept {
        cell_id: CellId,
        controller: AccountId,
        amount: Amount,
    },
}
