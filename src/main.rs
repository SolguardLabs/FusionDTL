mod amount;
mod capacity;
mod codec;
mod crypto;
mod delivery;
mod error;
mod exposure;
mod fusion;
mod ids;
mod ledger;
mod market;
mod operators;
mod participants;
mod risk;
mod routing;
mod runtime;
mod settlement;
mod treasury;

pub use amount::{Amount, Bps};
pub use capacity::{CapacityBook, CellCapacityPolicy};
pub use codec::canonical_bytes;
pub use crypto::{KeyPair, PublicIdentity, SignatureBytes, verify_signature};
pub use delivery::{
    DeliveryReceipt, ReceiptOrder, ReceiptOrderAuthorizationView, SettlementPacket,
    SignedReceiptOrder, SignedSettlementPacket,
};
pub use error::{FusionError, FusionResult};
pub use exposure::{CellExposure, ExposureBook};
pub use fusion::{CellConfig, LiquidityCell, ReceiptRecord};
pub use ids::{AccountId, AssetId, CellId, Digest, PacketId, ReceiptId, TxId};
pub use ledger::{AccountState, FusionLedger, JournalEntry, JournalOp};
pub use market::{AssetConfig, OracleBook, PriceObservation, VenueConfig};
pub use operators::{OperatorRegistry, OperatorRole, ProtocolConfig};
pub use participants::{
    Jurisdiction, ParticipantProfile, ParticipantStatus, ParticipantTier, ScreeningBook,
};
pub use risk::{RiskEngine, RiskLimits, RiskSnapshot};
pub use routing::{DeliveryLane, LaneBook, RelayerQuote};
pub use runtime::ScenarioReport;
pub use settlement::{SettlementCalendar, SettlementWindow};
pub use treasury::{FeeSchedule, InsuranceReserve, TreasuryBook};

fn main() {
    if let Err(error) = runtime::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
