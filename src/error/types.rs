use thiserror::Error;

use crate::{AccountId, Amount, AssetId, CellId, PacketId, ReceiptId, TxId};

pub type FusionResult<T> = Result<T, FusionError>;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FusionError {
    #[error("amount overflow")]
    AmountOverflow,
    #[error("amount underflow")]
    AmountUnderflow,
    #[error("division by zero")]
    DivisionByZero,
    #[error("zero amount")]
    ZeroAmount,
    #[error("basis points out of range: {0}")]
    BpsOutOfRange(u16),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("signature error: {0}")]
    Signature(String),
    #[error("invalid public key")]
    InvalidPublicKey,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("identity mismatch: {0}")]
    IdentityMismatch(AccountId),
    #[error("account not found: {0}")]
    AccountNotFound(AccountId),
    #[error("account already exists: {0}")]
    AccountAlreadyExists(AccountId),
    #[error("asset not found: {0}")]
    AssetNotFound(AssetId),
    #[error("asset already exists: {0}")]
    AssetAlreadyExists(AssetId),
    #[error("cell not found: {0}")]
    CellNotFound(CellId),
    #[error("cell already exists: {0}")]
    CellAlreadyExists(CellId),
    #[error("receipt not found: {0}")]
    ReceiptNotFound(ReceiptId),
    #[error("receipt already exists: {0}")]
    ReceiptAlreadyExists(ReceiptId),
    #[error("receipt already settled: {0}")]
    ReceiptSettled(ReceiptId),
    #[error("packet already processed: {0}")]
    PacketProcessed(PacketId),
    #[error("duplicate transaction: {0}")]
    DuplicateTransaction(TxId),
    #[error("unauthorized signer: expected {expected}, received {received}")]
    UnauthorizedSigner {
        expected: AccountId,
        received: AccountId,
    },
    #[error("nonce mismatch for {account}: expected {expected}, received {received}")]
    NonceMismatch {
        account: AccountId,
        expected: u64,
        received: u64,
    },
    #[error(
        "insufficient funds for {account} on {asset}: available {available}, required {required}"
    )]
    InsufficientFunds {
        account: AccountId,
        asset: AssetId,
        available: Amount,
        required: Amount,
    },
    #[error("policy violation: {0}")]
    Policy(String),
    #[error("conservation error for {asset}: expected {expected}, observed {observed}")]
    Conservation {
        asset: AssetId,
        expected: Amount,
        observed: Amount,
    },
}
