use ed25519_dalek::{Signature, Verifier};
use serde::{Serialize, Serializer};

use crate::{FusionError, FusionResult, PublicIdentity};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SignatureBytes(pub [u8; 64]);

impl Serialize for SignatureBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

pub fn verify_signature<T: Serialize>(
    signer: PublicIdentity,
    signature: SignatureBytes,
    domain: &str,
    payload: &T,
) -> FusionResult<()> {
    signer.verify_consistency()?;
    let message = super::identity::signed_message(domain, payload)?;
    let signature = Signature::from_bytes(&signature.0);
    signer
        .verifying_key()?
        .verify(&message, &signature)
        .map_err(|_| FusionError::InvalidSignature)
}
