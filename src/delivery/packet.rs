use serde::Serialize;

use crate::{
    AccountId, Amount, CellId, Digest, FusionError, FusionResult, KeyPair, PacketId,
    PublicIdentity, ReceiptId, SignatureBytes, TxId, verify_signature,
};

pub const SETTLEMENT_PACKET_DOMAIN: &str = "fusion-settlement-packet-v1";

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SettlementPacket {
    pub network_id: u32,
    pub packet_id: PacketId,
    pub cell_id: CellId,
    pub receipt_id: ReceiptId,
    pub beneficiary: AccountId,
    pub relayer: AccountId,
    pub relayer_fee: Amount,
    pub packet_nonce: u64,
    pub settlement_epoch: u64,
    pub receipt_digest: Digest,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SettlementPacketAuthorizationView {
    network_id: u32,
    packet_id: PacketId,
    cell_id: CellId,
    receipt_id: ReceiptId,
    beneficiary: AccountId,
    relayer: AccountId,
    relayer_fee: Amount,
    packet_nonce: u64,
    settlement_epoch: u64,
    receipt_digest: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignedSettlementPacket {
    pub signer: PublicIdentity,
    pub packet: SettlementPacket,
    pub signature: SignatureBytes,
}

impl SettlementPacket {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        network_id: u32,
        cell_id: CellId,
        receipt_id: ReceiptId,
        beneficiary: AccountId,
        relayer: AccountId,
        relayer_fee: Amount,
        packet_nonce: u64,
        settlement_epoch: u64,
        receipt_digest: Digest,
    ) -> FusionResult<Self> {
        let packet_id = PacketId::derive(network_id, cell_id, receipt_id, relayer, packet_nonce);
        Ok(Self {
            network_id,
            packet_id,
            cell_id,
            receipt_id,
            beneficiary,
            relayer,
            relayer_fee,
            packet_nonce,
            settlement_epoch,
            receipt_digest,
        })
    }

    pub fn authorization_view(self) -> SettlementPacketAuthorizationView {
        SettlementPacketAuthorizationView {
            network_id: self.network_id,
            packet_id: self.packet_id,
            cell_id: self.cell_id,
            receipt_id: self.receipt_id,
            beneficiary: self.beneficiary,
            relayer: self.relayer,
            relayer_fee: self.relayer_fee,
            packet_nonce: self.packet_nonce,
            settlement_epoch: self.settlement_epoch,
            receipt_digest: self.receipt_digest,
        }
    }
}

impl SignedSettlementPacket {
    pub fn sign(packet: SettlementPacket, key_pair: &KeyPair) -> FusionResult<Self> {
        let signer = key_pair.public_identity();
        if signer.account != packet.beneficiary {
            return Err(FusionError::UnauthorizedSigner {
                expected: packet.beneficiary,
                received: signer.account,
            });
        }
        let signature = key_pair.sign(SETTLEMENT_PACKET_DOMAIN, &packet.authorization_view())?;
        Ok(Self {
            signer,
            packet,
            signature,
        })
    }

    pub fn verify(&self) -> FusionResult<()> {
        if self.signer.account != self.packet.beneficiary {
            return Err(FusionError::UnauthorizedSigner {
                expected: self.packet.beneficiary,
                received: self.signer.account,
            });
        }
        verify_signature(
            self.signer,
            self.signature,
            SETTLEMENT_PACKET_DOMAIN,
            &self.packet.authorization_view(),
        )
    }

    pub fn tx_id(&self) -> FusionResult<TxId> {
        TxId::from_serializable("fusion-signed-settlement-packet-v1", self)
    }
}
