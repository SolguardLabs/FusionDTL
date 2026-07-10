use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use serde::Serialize;

use crate::{AccountId, Digest, FusionError, FusionResult, SignatureBytes, canonical_bytes};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PublicIdentity {
    pub account: AccountId,
    pub public_key: [u8; 32],
}

pub struct KeyPair {
    signing_key: SigningKey,
}

impl KeyPair {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(&seed),
        }
    }

    pub fn public_identity(&self) -> PublicIdentity {
        let public_key = self.signing_key.verifying_key().to_bytes();
        let account = Digest::from_parts("fusion-account-v1", &[&public_key]).into();
        PublicIdentity {
            account,
            public_key,
        }
    }

    pub fn sign<T: Serialize>(&self, domain: &str, payload: &T) -> FusionResult<SignatureBytes> {
        let message = signed_message(domain, payload)?;
        Ok(SignatureBytes(self.signing_key.sign(&message).to_bytes()))
    }
}

impl PublicIdentity {
    pub fn verifying_key(self) -> FusionResult<VerifyingKey> {
        VerifyingKey::from_bytes(&self.public_key).map_err(|_| FusionError::InvalidPublicKey)
    }

    pub fn verify_consistency(self) -> FusionResult<()> {
        let expected: AccountId =
            Digest::from_parts("fusion-account-v1", &[&self.public_key]).into();
        if expected != self.account {
            return Err(FusionError::IdentityMismatch(self.account));
        }
        Ok(())
    }
}

pub(crate) fn signed_message<T: Serialize>(domain: &str, payload: &T) -> FusionResult<Vec<u8>> {
    let bytes = canonical_bytes(payload)?;
    let digest = Digest::from_parts(domain, &[&bytes]);
    Ok(digest.bytes().to_vec())
}
