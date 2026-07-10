use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    AccountId, AccountState, Amount, AssetConfig, AssetId, CapacityBook, CellCapacityPolicy,
    CellConfig, CellId, DeliveryLane, DeliveryReceipt, Digest, ExposureBook, FeeSchedule,
    FusionError, FusionResult, InsuranceReserve, JournalEntry, JournalOp, LaneBook, LiquidityCell,
    OperatorRegistry, OperatorRole, OracleBook, ParticipantProfile, PriceObservation,
    ProtocolConfig, PublicIdentity, ReceiptId, ReceiptRecord, RelayerQuote, RiskEngine, RiskLimits,
    ScreeningBook, SettlementCalendar, SettlementWindow, SignedReceiptOrder,
    SignedSettlementPacket, TreasuryBook, TxId, VenueConfig,
};

#[derive(Clone, Debug, Serialize)]
pub struct FusionLedger {
    network_id: u32,
    assets: BTreeMap<AssetId, AssetConfig>,
    accounts: BTreeMap<AccountId, AccountState>,
    total_supply: BTreeMap<AssetId, Amount>,
    cells: BTreeMap<CellId, LiquidityCell>,
    receipts: BTreeMap<ReceiptId, DeliveryReceipt>,
    receipt_records: BTreeMap<ReceiptId, ReceiptRecord>,
    processed_packets: BTreeSet<crate::PacketId>,
    seen_transactions: BTreeSet<TxId>,
    oracle_book: OracleBook,
    venue: VenueConfig,
    risk_engine: RiskEngine,
    screening_book: ScreeningBook,
    settlement_calendar: SettlementCalendar,
    operator_registry: OperatorRegistry,
    lane_book: LaneBook,
    treasury_book: TreasuryBook,
    exposure_book: ExposureBook,
    capacity_book: CapacityBook,
    journal: Vec<JournalEntry>,
}

#[derive(Serialize)]
struct LedgerDigestView<'a> {
    network_id: u32,
    assets: &'a BTreeMap<AssetId, AssetConfig>,
    accounts: &'a BTreeMap<AccountId, AccountState>,
    total_supply: &'a BTreeMap<AssetId, Amount>,
    cells: &'a BTreeMap<CellId, LiquidityCell>,
    receipts: &'a BTreeMap<ReceiptId, DeliveryReceipt>,
    receipt_records: &'a BTreeMap<ReceiptId, ReceiptRecord>,
    processed_packets: &'a BTreeSet<crate::PacketId>,
    seen_transactions: &'a BTreeSet<TxId>,
    oracle_book: &'a OracleBook,
    venue: &'a VenueConfig,
    risk_engine: &'a RiskEngine,
    screening_book: &'a ScreeningBook,
    settlement_calendar: &'a SettlementCalendar,
    operator_registry: &'a OperatorRegistry,
    lane_book: &'a LaneBook,
    treasury_book: &'a TreasuryBook,
    exposure_book: &'a ExposureBook,
    capacity_book: &'a CapacityBook,
    journal_len: usize,
}

impl FusionLedger {
    pub fn new(network_id: u32, venue: VenueConfig) -> Self {
        Self {
            network_id,
            assets: BTreeMap::new(),
            accounts: BTreeMap::new(),
            total_supply: BTreeMap::new(),
            cells: BTreeMap::new(),
            receipts: BTreeMap::new(),
            receipt_records: BTreeMap::new(),
            processed_packets: BTreeSet::new(),
            seen_transactions: BTreeSet::new(),
            oracle_book: OracleBook::default(),
            venue,
            risk_engine: RiskEngine::default(),
            screening_book: ScreeningBook::default(),
            settlement_calendar: SettlementCalendar::default(),
            operator_registry: OperatorRegistry::default(),
            lane_book: LaneBook::default(),
            treasury_book: TreasuryBook::default(),
            exposure_book: ExposureBook::default(),
            capacity_book: CapacityBook::default(),
            journal: Vec::new(),
        }
    }

    pub const fn network_id(&self) -> u32 {
        self.network_id
    }

    pub fn set_risk_limits(&mut self, limits: RiskLimits) {
        self.risk_engine.set_limits(limits);
    }

    pub fn publish_price(&mut self, observation: PriceObservation) {
        self.oracle_book.publish(observation);
    }

    pub fn configure_protocol(&mut self, config: ProtocolConfig) -> FusionResult<TxId> {
        self.account(config.admin)?;
        self.operator_registry.configure(config);
        let tx_id = TxId::from_serializable(
            "fusion-protocol-config-tx-v1",
            &(self.network_id, config, self.journal.len() as u64),
        )?;
        self.append_journal(tx_id, JournalOp::ProtocolConfigured { config })?;
        Ok(tx_id)
    }

    pub fn grant_operator_role(
        &mut self,
        account: AccountId,
        role: OperatorRole,
    ) -> FusionResult<TxId> {
        self.account(account)?;
        self.operator_registry.grant_role(account, role);
        let tx_id = TxId::from_serializable(
            "fusion-operator-role-tx-v1",
            &(self.network_id, account, role, self.journal.len() as u64),
        )?;
        self.append_journal(tx_id, JournalOp::OperatorRoleGranted { account, role })?;
        Ok(tx_id)
    }

    pub fn attest_participant(&mut self, profile: ParticipantProfile) -> FusionResult<TxId> {
        self.account(profile.account)?;
        self.screening_book.attest(profile);
        let tx_id = TxId::from_serializable(
            "fusion-participant-attestation-tx-v1",
            &(self.network_id, profile, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::ParticipantAttested {
                account: profile.account,
                tier: profile.tier,
                jurisdiction: profile.jurisdiction,
                status: profile.status,
                expires_at_epoch: profile.expires_at_epoch,
            },
        )?;
        Ok(tx_id)
    }

    pub fn register_settlement_window(&mut self, window: SettlementWindow) -> FusionResult<TxId> {
        let window_id = self.settlement_calendar.register_window(window)?;
        let tx_id = TxId::from_serializable(
            "fusion-settlement-window-tx-v1",
            &(self.network_id, window, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::SettlementWindowRegistered {
                window_id,
                opens_at_epoch: window.opens_at_epoch,
                closes_at_epoch: window.closes_at_epoch,
            },
        )?;
        Ok(tx_id)
    }

    pub fn register_delivery_lane(&mut self, lane: DeliveryLane) -> FusionResult<TxId> {
        self.cell(lane.source_cell)?;
        self.cell(lane.payout_cell)?;
        self.asset_config(lane.asset)?;
        let lane_id = self.lane_book.register_lane(lane)?;
        let tx_id = TxId::from_serializable(
            "fusion-delivery-lane-tx-v1",
            &(self.network_id, lane, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::DeliveryLaneRegistered {
                lane_id,
                source_cell: lane.source_cell,
                payout_cell: lane.payout_cell,
                asset: lane.asset,
            },
        )?;
        Ok(tx_id)
    }

    pub fn submit_relayer_quote(&mut self, quote: RelayerQuote) -> FusionResult<TxId> {
        self.account(quote.relayer)?;
        self.operator_registry
            .require_role(quote.relayer, OperatorRole::Relayer)?;
        let quote_digest = self.lane_book.submit_quote(quote)?;
        let tx_id = TxId::from_serializable(
            "fusion-relayer-quote-tx-v1",
            &(self.network_id, quote, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::RelayerQuoteSubmitted {
                lane_id: quote.lane_id,
                relayer: quote.relayer,
                quote_digest,
                relayer_fee: quote.relayer_fee,
            },
        )?;
        Ok(tx_id)
    }

    pub fn configure_fee_schedule(&mut self, schedule: FeeSchedule) -> FusionResult<TxId> {
        self.account(schedule.treasury)?;
        self.operator_registry
            .require_role(schedule.treasury, OperatorRole::Treasury)?;
        self.treasury_book.configure_schedule(schedule);
        let tx_id = TxId::from_serializable(
            "fusion-fee-schedule-tx-v1",
            &(self.network_id, schedule, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::FeeScheduleConfigured {
                treasury: schedule.treasury,
                protocol_fee_bps: schedule.protocol_fee_bps,
                insurance_fee_bps: schedule.insurance_fee_bps,
            },
        )?;
        Ok(tx_id)
    }

    pub fn configure_insurance_reserve(&mut self, reserve: InsuranceReserve) -> FusionResult<TxId> {
        self.asset_config(reserve.asset)?;
        self.treasury_book.configure_insurance(reserve);
        let tx_id = TxId::from_serializable(
            "fusion-insurance-reserve-tx-v1",
            &(self.network_id, reserve, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::InsuranceReserveConfigured {
                asset: reserve.asset,
                balance: reserve.balance,
                floor: reserve.floor,
            },
        )?;
        Ok(tx_id)
    }

    pub fn configure_cell_capacity(&mut self, policy: CellCapacityPolicy) -> FusionResult<TxId> {
        let cell = self.cell(policy.cell_id)?;
        if cell.config.reserve_asset != policy.asset {
            return Err(FusionError::Policy(
                "capacity policy asset mismatch".to_owned(),
            ));
        }
        let policy_id = self.capacity_book.configure(policy);
        let tx_id = TxId::from_serializable(
            "fusion-cell-capacity-tx-v1",
            &(self.network_id, policy, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::CellCapacityConfigured {
                policy_id,
                cell_id: policy.cell_id,
                soft_cap: policy.soft_cap,
                hard_cap: policy.hard_cap,
            },
        )?;
        Ok(tx_id)
    }

    pub fn register_asset(&mut self, config: AssetConfig) -> FusionResult<()> {
        if self.assets.contains_key(&config.id) {
            return Err(FusionError::AssetAlreadyExists(config.id));
        }
        self.total_supply
            .entry(config.id)
            .or_insert_with(Amount::zero);
        self.assets.insert(config.id, config);
        Ok(())
    }

    pub fn register_account(&mut self, identity: PublicIdentity) -> FusionResult<()> {
        identity.verify_consistency()?;
        if self.accounts.contains_key(&identity.account) {
            return Err(FusionError::AccountAlreadyExists(identity.account));
        }
        self.accounts
            .insert(identity.account, AccountState::new(identity));
        Ok(())
    }

    pub fn register_cell(&mut self, config: CellConfig) -> FusionResult<TxId> {
        self.account(config.controller)?;
        self.asset_config(config.reserve_asset)?;
        if self.cells.contains_key(&config.cell_id) {
            return Err(FusionError::CellAlreadyExists(config.cell_id));
        }
        self.cells
            .insert(config.cell_id, LiquidityCell::new(config));
        let tx_id = TxId::from_serializable(
            "fusion-register-cell-tx-v1",
            &(self.network_id, config, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::CellRegistered {
                cell_id: config.cell_id,
                controller: config.controller,
                reserve_asset: config.reserve_asset,
            },
        )?;
        Ok(tx_id)
    }

    pub fn credit_genesis(
        &mut self,
        account: AccountId,
        asset: AssetId,
        amount: Amount,
    ) -> FusionResult<TxId> {
        if amount.is_zero() {
            return Err(FusionError::ZeroAmount);
        }
        self.asset_config(asset)?;
        let mut candidate = self.clone();
        candidate.credit(account, asset, amount)?;
        let supply = candidate.total_supply_of(asset).checked_add(amount)?;
        candidate.total_supply.insert(asset, supply);
        let tx_id = TxId::from_serializable(
            "fusion-genesis-credit-v1",
            &(
                self.network_id,
                account,
                asset,
                amount,
                self.journal.len() as u64,
            ),
        )?;
        candidate.append_journal(
            tx_id,
            JournalOp::GenesisCredit {
                account,
                asset,
                amount,
            },
        )?;
        candidate.verify_conservation(asset)?;
        *self = candidate;
        Ok(tx_id)
    }

    pub fn deposit_cell(
        &mut self,
        owner: AccountId,
        cell_id: CellId,
        amount: Amount,
    ) -> FusionResult<TxId> {
        if amount.is_zero() {
            return Err(FusionError::ZeroAmount);
        }
        let mut candidate = self.clone();
        let asset = candidate.cell(cell_id)?.config.reserve_asset;
        candidate.debit(owner, asset, amount)?;
        candidate.cell_mut(cell_id)?.deposit(amount)?;
        let tx_id = TxId::from_serializable(
            "fusion-cell-deposit-tx-v1",
            &(
                self.network_id,
                owner,
                cell_id,
                amount,
                self.journal.len() as u64,
            ),
        )?;
        candidate.append_journal(
            tx_id,
            JournalOp::CellDeposit {
                cell_id,
                owner,
                amount,
            },
        )?;
        candidate.verify_conservation(asset)?;
        *self = candidate;
        Ok(tx_id)
    }

    pub fn issue_receipt(&mut self, signed: &SignedReceiptOrder) -> FusionResult<TxId> {
        let mut candidate = self.clone();
        let tx_id = candidate.issue_receipt_inner(signed)?;
        let asset = signed.order.asset;
        candidate.verify_conservation(asset)?;
        *self = candidate;
        Ok(tx_id)
    }

    pub fn settle_packet(&mut self, signed: &SignedSettlementPacket) -> FusionResult<TxId> {
        let mut candidate = self.clone();
        let tx_id = candidate.settle_packet_inner(signed)?;
        let receipt = candidate
            .receipts
            .get(&signed.packet.receipt_id)
            .copied()
            .ok_or(FusionError::ReceiptNotFound(signed.packet.receipt_id))?;
        candidate.verify_conservation(receipt.asset)?;
        *self = candidate;
        Ok(tx_id)
    }

    pub fn sweep_cell_surplus(
        &mut self,
        controller: AccountId,
        cell_id: CellId,
        amount: Amount,
    ) -> FusionResult<TxId> {
        let mut candidate = self.clone();
        let asset = candidate.cell(cell_id)?.config.reserve_asset;
        if candidate.cell(cell_id)?.config.controller != controller {
            return Err(FusionError::UnauthorizedSigner {
                expected: candidate.cell(cell_id)?.config.controller,
                received: controller,
            });
        }
        candidate.cell_mut(cell_id)?.sweep(amount)?;
        candidate.credit(controller, asset, amount)?;
        let tx_id = TxId::from_serializable(
            "fusion-cell-surplus-sweep-tx-v1",
            &(
                self.network_id,
                controller,
                cell_id,
                amount,
                self.journal.len() as u64,
            ),
        )?;
        candidate.append_journal(
            tx_id,
            JournalOp::CellSurplusSwept {
                cell_id,
                controller,
                amount,
            },
        )?;
        candidate.verify_conservation(asset)?;
        *self = candidate;
        Ok(tx_id)
    }

    pub fn balance_of(&self, account: AccountId, asset: AssetId) -> FusionResult<Amount> {
        Ok(self.account(account)?.balance_of(asset))
    }

    pub fn receipt_nonce(&self, account: AccountId) -> FusionResult<u64> {
        Ok(self.account(account)?.next_receipt_nonce)
    }

    pub fn packet_nonce(&self, account: AccountId) -> FusionResult<u64> {
        Ok(self.account(account)?.next_packet_nonce)
    }

    pub fn total_supply_of(&self, asset: AssetId) -> Amount {
        self.total_supply
            .get(&asset)
            .copied()
            .unwrap_or_else(Amount::zero)
    }

    pub fn cell(&self, cell_id: CellId) -> FusionResult<&LiquidityCell> {
        self.cells
            .get(&cell_id)
            .ok_or(FusionError::CellNotFound(cell_id))
    }

    pub fn receipt(&self, receipt_id: ReceiptId) -> FusionResult<DeliveryReceipt> {
        self.receipts
            .get(&receipt_id)
            .copied()
            .ok_or(FusionError::ReceiptNotFound(receipt_id))
    }

    pub fn receipt_count(&self) -> usize {
        self.receipts.len()
    }

    pub fn processed_packet_count(&self) -> usize {
        self.processed_packets.len()
    }

    pub fn oracle_market_count(&self) -> usize {
        self.oracle_book.market_count()
    }

    pub fn journal(&self) -> &[JournalEntry] {
        &self.journal
    }

    pub fn state_digest(&self) -> FusionResult<Digest> {
        Digest::from_serializable(
            "fusion-ledger-state-v1",
            &LedgerDigestView {
                network_id: self.network_id,
                assets: &self.assets,
                accounts: &self.accounts,
                total_supply: &self.total_supply,
                cells: &self.cells,
                receipts: &self.receipts,
                receipt_records: &self.receipt_records,
                processed_packets: &self.processed_packets,
                seen_transactions: &self.seen_transactions,
                oracle_book: &self.oracle_book,
                venue: &self.venue,
                risk_engine: &self.risk_engine,
                screening_book: &self.screening_book,
                settlement_calendar: &self.settlement_calendar,
                operator_registry: &self.operator_registry,
                lane_book: &self.lane_book,
                treasury_book: &self.treasury_book,
                exposure_book: &self.exposure_book,
                capacity_book: &self.capacity_book,
                journal_len: self.journal.len(),
            },
        )
    }

    pub fn is_conserved(&self, asset: AssetId) -> FusionResult<bool> {
        self.verify_conservation(asset)?;
        Ok(true)
    }

    pub fn operator_count(&self) -> usize {
        self.operator_registry.operator_count()
    }

    pub fn role_assignment_count(&self) -> usize {
        self.operator_registry.role_assignment_count()
    }

    pub fn delivery_lane_count(&self) -> usize {
        self.lane_book.lane_count()
    }

    pub fn relayer_quote_count(&self) -> usize {
        self.lane_book.quote_count()
    }

    pub fn treasury_asset_count(&self) -> usize {
        self.treasury_book.accrued_asset_count()
    }

    pub fn exposure_cell_count(&self) -> usize {
        self.exposure_book.cell_count()
    }

    pub fn participant_profile_count(&self) -> usize {
        self.screening_book.profile_count()
    }

    pub fn active_profile_count(&self) -> usize {
        self.screening_book.active_profile_count()
    }

    pub fn settlement_window_count(&self) -> usize {
        self.settlement_calendar.window_count()
    }

    pub fn capacity_policy_count(&self) -> usize {
        self.capacity_book.policy_count()
    }

    fn issue_receipt_inner(&mut self, signed: &SignedReceiptOrder) -> FusionResult<TxId> {
        signed.verify()?;
        let order = signed.order;
        self.operator_registry.ensure_not_paused()?;
        self.operator_registry
            .require_role(order.issuer, OperatorRole::Issuer)?;
        self.screening_book
            .require_active(order.issuer, order.maturity_epoch)?;
        self.screening_book
            .require_active(order.beneficiary, order.maturity_epoch)?;
        self.settlement_calendar.ensure_open(order.maturity_epoch)?;
        if order.network_id != self.network_id {
            return Err(FusionError::Policy("network mismatch".to_owned()));
        }
        if self.receipt_nonce(order.issuer)? != order.owner_nonce {
            return Err(FusionError::NonceMismatch {
                account: order.issuer,
                expected: self.receipt_nonce(order.issuer)?,
                received: order.owner_nonce,
            });
        }
        let cell = self.cell(order.source_cell)?;
        if cell.config.reserve_asset != order.asset {
            return Err(FusionError::Policy("receipt asset mismatch".to_owned()));
        }
        self.capacity_book.ensure_issue_allowed(
            order.source_cell,
            order.asset,
            order.amount,
            cell.pending_liability,
        )?;
        if !self.asset_config(order.asset)?.settlement_enabled {
            return Err(FusionError::Policy("asset settlement disabled".to_owned()));
        }
        let receipt = order.receipt();
        if self.receipts.contains_key(&receipt.receipt_id) {
            return Err(FusionError::ReceiptAlreadyExists(receipt.receipt_id));
        }
        let tx_id = signed.tx_id()?;
        if self.seen_transactions.contains(&tx_id) {
            return Err(FusionError::DuplicateTransaction(tx_id));
        }
        self.debit(order.issuer, order.asset, order.amount)?;
        self.cell_mut(order.source_cell)?.deposit(order.amount)?;
        self.cell_mut(order.source_cell)?
            .register_receipt(order.amount)?;
        self.exposure_book
            .record_issue(order.source_cell, order.amount)?;
        self.account_mut(order.issuer)?.advance_receipt_nonce()?;
        self.receipts.insert(receipt.receipt_id, receipt);
        self.receipt_records.insert(
            receipt.receipt_id,
            ReceiptRecord {
                receipt_id: receipt.receipt_id,
                source_cell: order.source_cell,
                issuer: order.issuer,
                settled: false,
            },
        );
        self.seen_transactions.insert(tx_id);
        self.append_journal(
            tx_id,
            JournalOp::ReceiptIssued {
                receipt_id: receipt.receipt_id,
                source_cell: order.source_cell,
                issuer: order.issuer,
                beneficiary: order.beneficiary,
                amount: order.amount,
            },
        )?;
        Ok(tx_id)
    }

    fn settle_packet_inner(&mut self, signed: &SignedSettlementPacket) -> FusionResult<TxId> {
        signed.verify()?;
        let packet = signed.packet;
        self.operator_registry.ensure_not_paused()?;
        self.operator_registry
            .require_role(packet.beneficiary, OperatorRole::Beneficiary)?;
        self.operator_registry
            .require_role(packet.relayer, OperatorRole::Relayer)?;
        self.screening_book
            .require_active(packet.beneficiary, packet.settlement_epoch)?;
        self.screening_book
            .require_active(packet.relayer, packet.settlement_epoch)?;
        self.settlement_calendar
            .ensure_open(packet.settlement_epoch)?;
        if packet.network_id != self.network_id {
            return Err(FusionError::Policy("network mismatch".to_owned()));
        }
        if self.processed_packets.contains(&packet.packet_id) {
            return Err(FusionError::PacketProcessed(packet.packet_id));
        }
        let receipt = self.receipt(packet.receipt_id)?;
        let mut record = *self
            .receipt_records
            .get(&packet.receipt_id)
            .ok_or(FusionError::ReceiptNotFound(packet.receipt_id))?;
        if record.settled {
            return Err(FusionError::ReceiptSettled(packet.receipt_id));
        }
        if packet.beneficiary != receipt.beneficiary {
            return Err(FusionError::UnauthorizedSigner {
                expected: receipt.beneficiary,
                received: packet.beneficiary,
            });
        }
        if packet.receipt_digest != receipt.digest()? {
            return Err(FusionError::Policy("receipt digest mismatch".to_owned()));
        }
        if packet.settlement_epoch < receipt.maturity_epoch {
            return Err(FusionError::Policy("receipt is not mature".to_owned()));
        }
        if packet.settlement_epoch < self.risk_engine.limits().current_epoch {
            return Err(FusionError::Policy("stale settlement packet".to_owned()));
        }
        if self.packet_nonce(packet.beneficiary)? != packet.packet_nonce {
            return Err(FusionError::NonceMismatch {
                account: packet.beneficiary,
                expected: self.packet_nonce(packet.beneficiary)?,
                received: packet.packet_nonce,
            });
        }
        let payout_cell = self.cell(packet.cell_id)?;
        if payout_cell.config.reserve_asset != receipt.asset {
            return Err(FusionError::Policy("payout cell asset mismatch".to_owned()));
        }
        let lane = self.lane_book.resolve_lane(
            record.source_cell,
            packet.cell_id,
            receipt.asset,
            receipt.amount,
            packet.relayer_fee,
        )?;
        let risk = self.risk_engine.evaluate_delivery(
            packet.packet_id,
            payout_cell,
            receipt.amount,
            packet.relayer_fee,
        )?;
        let protocol_fee = self.treasury_book.protocol_fee(packet.relayer_fee)?;
        let insurance_fee = self.treasury_book.insurance_fee(packet.relayer_fee)?;
        let tx_id = signed.tx_id()?;
        if self.seen_transactions.contains(&tx_id) {
            return Err(FusionError::DuplicateTransaction(tx_id));
        }
        let beneficiary_amount = receipt.amount.checked_sub(packet.relayer_fee)?;
        self.cell_mut(packet.cell_id)?
            .pay_delivery(receipt.amount)?;
        self.cell_mut(record.source_cell)?
            .settle_liability(receipt.amount)?;
        self.credit(packet.beneficiary, receipt.asset, beneficiary_amount)?;
        if !packet.relayer_fee.is_zero() {
            self.credit(packet.relayer, receipt.asset, packet.relayer_fee)?;
        }
        self.treasury_book.accrue(receipt.asset, protocol_fee)?;
        self.treasury_book
            .absorb_insurance_fee(receipt.asset, insurance_fee)?;
        self.exposure_book
            .record_settlement(record.source_cell, packet.cell_id, receipt.amount)?;
        self.account_mut(packet.beneficiary)?
            .advance_packet_nonce()?;
        record.settled = true;
        self.receipt_records.insert(packet.receipt_id, record);
        self.processed_packets.insert(packet.packet_id);
        self.seen_transactions.insert(tx_id);
        self.append_journal(
            tx_id,
            JournalOp::PacketSettled {
                packet_id: packet.packet_id,
                receipt_id: packet.receipt_id,
                lane_id: lane.lane_id,
                payout_cell: packet.cell_id,
                source_cell: record.source_cell,
                beneficiary: packet.beneficiary,
                beneficiary_amount,
                relayer: packet.relayer,
                relayer_fee: packet.relayer_fee,
                protocol_fee,
                insurance_fee,
                risk: Box::new(risk),
            },
        )?;
        Ok(tx_id)
    }

    fn append_journal(&mut self, tx_id: TxId, op: JournalOp) -> FusionResult<()> {
        let entry = JournalEntry {
            sequence: self.journal.len() as u64,
            tx_id,
            op,
            state_digest: self.state_digest()?,
        };
        self.journal.push(entry);
        Ok(())
    }

    fn asset_config(&self, asset: AssetId) -> FusionResult<AssetConfig> {
        self.assets
            .get(&asset)
            .copied()
            .ok_or(FusionError::AssetNotFound(asset))
    }

    fn account(&self, account: AccountId) -> FusionResult<&AccountState> {
        self.accounts
            .get(&account)
            .ok_or(FusionError::AccountNotFound(account))
    }

    fn account_mut(&mut self, account: AccountId) -> FusionResult<&mut AccountState> {
        self.accounts
            .get_mut(&account)
            .ok_or(FusionError::AccountNotFound(account))
    }

    fn cell_mut(&mut self, cell_id: CellId) -> FusionResult<&mut LiquidityCell> {
        self.cells
            .get_mut(&cell_id)
            .ok_or(FusionError::CellNotFound(cell_id))
    }

    fn credit(&mut self, account: AccountId, asset: AssetId, amount: Amount) -> FusionResult<()> {
        self.account_mut(account)?.credit(asset, amount)
    }

    fn debit(&mut self, account: AccountId, asset: AssetId, amount: Amount) -> FusionResult<()> {
        self.account_mut(account)?.debit(account, asset, amount)
    }

    fn verify_conservation(&self, asset: AssetId) -> FusionResult<()> {
        let account_total = self
            .accounts
            .values()
            .try_fold(Amount::zero(), |accumulator, account| {
                accumulator.checked_add(account.balance_of(asset))
            })?;
        let cell_total = self
            .cells
            .values()
            .filter(|cell| cell.config.reserve_asset == asset)
            .try_fold(Amount::zero(), |accumulator, cell| {
                accumulator.checked_add(cell.reserve_balance)
            })?;
        let observed = account_total.checked_add(cell_total)?;
        let expected = self.total_supply_of(asset);
        if observed != expected {
            return Err(FusionError::Conservation {
                asset,
                expected,
                observed,
            });
        }
        Ok(())
    }
}
