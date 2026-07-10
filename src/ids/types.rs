use serde::{Deserialize, Serialize, Serializer};

use crate::{FusionResult, canonical_bytes};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct AccountId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct AssetId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct CellId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct ReceiptId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct PacketId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct TxId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct Digest([u8; 32]);

macro_rules! id_type {
    ($name:ident) => {
        impl $name {
            pub const fn from_bytes(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }

            pub const fn bytes(self) -> [u8; 32] {
                self.0
            }

            pub fn from_serializable<T: Serialize>(domain: &str, value: &T) -> FusionResult<Self> {
                let digest = Digest::from_serializable(domain, value)?;
                Ok(Self(digest.bytes()))
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&hex::encode(self.0))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "{}", hex::encode(self.0))
            }
        }
    };
}

id_type!(AccountId);
id_type!(AssetId);
id_type!(CellId);
id_type!(ReceiptId);
id_type!(PacketId);
id_type!(TxId);

impl Digest {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }

    pub fn from_parts(domain: &str, parts: &[&[u8]]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(domain.as_bytes());
        for part in parts {
            hasher.update(&(part.len() as u64).to_be_bytes());
            hasher.update(part);
        }
        Self(*hasher.finalize().as_bytes())
    }

    pub fn from_serializable<T: Serialize>(domain: &str, value: &T) -> FusionResult<Self> {
        let bytes = canonical_bytes(value)?;
        Ok(Self::from_parts(domain, &[&bytes]))
    }
}

impl Serialize for Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl std::fmt::Display for Digest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", hex::encode(self.0))
    }
}

impl AssetId {
    pub fn derive(symbol: &str, decimals: u8) -> Self {
        Digest::from_parts("fusion-asset-v1", &[symbol.as_bytes(), &[decimals]]).into()
    }
}

impl CellId {
    pub fn derive(controller: AccountId, reserve_asset: AssetId, lane: u16, salt: Digest) -> Self {
        Digest::from_parts(
            "fusion-cell-v1",
            &[
                &controller.bytes(),
                &reserve_asset.bytes(),
                &lane.to_be_bytes(),
                &salt.bytes(),
            ],
        )
        .into()
    }
}

impl ReceiptId {
    pub fn derive(
        network_id: u32,
        beneficiary: AccountId,
        asset: AssetId,
        amount: crate::Amount,
        owner_nonce: u64,
        route_digest: Digest,
    ) -> Self {
        Digest::from_parts(
            "fusion-receipt-v1",
            &[
                &network_id.to_be_bytes(),
                &beneficiary.bytes(),
                &asset.bytes(),
                &amount.units().to_be_bytes(),
                &owner_nonce.to_be_bytes(),
                &route_digest.bytes(),
            ],
        )
        .into()
    }
}

impl PacketId {
    pub fn derive(
        network_id: u32,
        cell_id: CellId,
        receipt_id: ReceiptId,
        relayer: AccountId,
        packet_nonce: u64,
    ) -> Self {
        Digest::from_parts(
            "fusion-packet-v1",
            &[
                &network_id.to_be_bytes(),
                &cell_id.bytes(),
                &receipt_id.bytes(),
                &relayer.bytes(),
                &packet_nonce.to_be_bytes(),
            ],
        )
        .into()
    }
}

impl From<Digest> for AccountId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for AssetId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for CellId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for ReceiptId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for PacketId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for TxId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}
