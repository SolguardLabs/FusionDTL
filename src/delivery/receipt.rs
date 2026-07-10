use serde::Serialize;

use crate::{
    AccountId, Amount, AssetId, CellId, Digest, FusionError, FusionResult, KeyPair, PublicIdentity,
    ReceiptId, SignatureBytes, TxId, verify_signature,
};

pub const RECEIPT_ORDER_DOMAIN: &str = "fusion-receipt-order-v1";

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReceiptOrder {
    pub network_id: u32,
    pub source_cell: CellId,
    pub issuer: AccountId,
    pub beneficiary: AccountId,
    pub asset: AssetId,
    pub amount: Amount,
    pub owner_nonce: u64,
    pub maturity_epoch: u64,
    pub route_digest: Digest,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReceiptOrderAuthorizationView {
    network_id: u32,
    source_cell: CellId,
    issuer: AccountId,
    beneficiary: AccountId,
    asset: AssetId,
    amount: Amount,
    owner_nonce: u64,
    maturity_epoch: u64,
    route_digest: Digest,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DeliveryReceipt {
    pub receipt_id: ReceiptId,
    pub network_id: u32,
    pub beneficiary: AccountId,
    pub asset: AssetId,
    pub amount: Amount,
    pub owner_nonce: u64,
    pub maturity_epoch: u64,
    pub route_digest: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignedReceiptOrder {
    pub signer: PublicIdentity,
    pub order: ReceiptOrder,
    pub signature: SignatureBytes,
}

impl ReceiptOrder {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        network_id: u32,
        source_cell: CellId,
        issuer: AccountId,
        beneficiary: AccountId,
        asset: AssetId,
        amount: Amount,
        owner_nonce: u64,
        maturity_epoch: u64,
        route_digest: Digest,
    ) -> FusionResult<Self> {
        if amount.is_zero() {
            return Err(FusionError::ZeroAmount);
        }
        Ok(Self {
            network_id,
            source_cell,
            issuer,
            beneficiary,
            asset,
            amount,
            owner_nonce,
            maturity_epoch,
            route_digest,
        })
    }

    pub fn authorization_view(self) -> ReceiptOrderAuthorizationView {
        ReceiptOrderAuthorizationView {
            network_id: self.network_id,
            source_cell: self.source_cell,
            issuer: self.issuer,
            beneficiary: self.beneficiary,
            asset: self.asset,
            amount: self.amount,
            owner_nonce: self.owner_nonce,
            maturity_epoch: self.maturity_epoch,
            route_digest: self.route_digest,
        }
    }

    pub fn receipt(self) -> DeliveryReceipt {
        let receipt_id = ReceiptId::derive(
            self.network_id,
            self.beneficiary,
            self.asset,
            self.amount,
            self.owner_nonce,
            self.route_digest,
        );
        DeliveryReceipt {
            receipt_id,
            network_id: self.network_id,
            beneficiary: self.beneficiary,
            asset: self.asset,
            amount: self.amount,
            owner_nonce: self.owner_nonce,
            maturity_epoch: self.maturity_epoch,
            route_digest: self.route_digest,
        }
    }
}

impl DeliveryReceipt {
    pub fn digest(self) -> FusionResult<Digest> {
        Digest::from_serializable("fusion-delivery-receipt-v1", &self)
    }
}

impl SignedReceiptOrder {
    pub fn sign(order: ReceiptOrder, key_pair: &KeyPair) -> FusionResult<Self> {
        let signer = key_pair.public_identity();
        if signer.account != order.issuer {
            return Err(FusionError::UnauthorizedSigner {
                expected: order.issuer,
                received: signer.account,
            });
        }
        let signature = key_pair.sign(RECEIPT_ORDER_DOMAIN, &order.authorization_view())?;
        Ok(Self {
            signer,
            order,
            signature,
        })
    }

    pub fn verify(&self) -> FusionResult<()> {
        if self.signer.account != self.order.issuer {
            return Err(FusionError::UnauthorizedSigner {
                expected: self.order.issuer,
                received: self.signer.account,
            });
        }
        verify_signature(
            self.signer,
            self.signature,
            RECEIPT_ORDER_DOMAIN,
            &self.order.authorization_view(),
        )
    }

    pub fn tx_id(&self) -> FusionResult<TxId> {
        TxId::from_serializable("fusion-signed-receipt-order-v1", self)
    }
}
