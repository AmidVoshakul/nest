//! Cryptographic manifest signing and verification
//!
//! Uses Ed25519 digital signatures to guarantee integrity and authenticity
//! of agent manifest files. Prevents tampering and unauthorized modifications.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A signed manifest with Ed25519 signature
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct SignedManifest {
    /// Serialized manifest content
    pub content: Vec<u8>,

    /// Ed25519 signature over content
    pub signature: Vec<u8>,

    /// Public key of the signer
    pub public_key: Vec<u8>,
}

/// Manifest signer that holds private key
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ManifestSigner {
    #[zeroize(skip)]
    signing_key: SigningKey,
}

/// Manifest verifier that holds trusted public keys
#[derive(Debug, Clone)]
pub struct ManifestVerifier {
    trusted_keys: Vec<Vec<u8>>,
}

impl SignedManifest {
    /// Create a new signed manifest
    pub fn new(content: &[u8], signer: &ManifestSigner) -> Self {
        let signature = signer.signing_key.sign(content).to_bytes().to_vec();

        Self {
            content: content.to_vec(),
            signature,
            public_key: signer.public_key(),
        }
    }

    /// Verify this manifest's signature
    pub fn verify(&self) -> Result<(), crate::error::Error> {
        if self.public_key.len() != 32 {
            return Err(crate::error::Error::Unknown(
                "Invalid public key length".into(),
            ));
        }

        if self.signature.len() != 64 {
            return Err(crate::error::Error::Unknown(
                "Invalid signature length".into(),
            ));
        }

        let mut pub_key_bytes = [0u8; 32];
        pub_key_bytes.copy_from_slice(&self.public_key);

        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(&self.signature);

        let public_key = VerifyingKey::from_bytes(&pub_key_bytes)
            .map_err(|_| crate::error::Error::Unknown("Invalid public key".into()))?;

        let signature = Signature::from_bytes(&sig_bytes);

        public_key
            .verify(&self.content, &signature)
            .map_err(|_| crate::error::Error::Unknown("Invalid manifest signature".into()))?;

        Ok(())
    }
}

impl ManifestSigner {
    /// Generate new random signing key
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);

        Self { signing_key }
    }

    /// Create signer from existing private key
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(bytes),
        }
    }

    /// Get public key for this signer
    pub fn public_key(&self) -> Vec<u8> {
        self.signing_key.verifying_key().to_bytes().to_vec()
    }

    /// Sign content
    pub fn sign(&self, content: &[u8]) -> Vec<u8> {
        self.signing_key.sign(content).to_bytes().to_vec()
    }
}

impl ManifestVerifier {
    /// Create new verifier with trusted public keys
    pub fn new(trusted_keys: Vec<Vec<u8>>) -> Self {
        Self { trusted_keys }
    }

    /// Add a trusted public key
    pub fn add_trusted_key(&mut self, key: &[u8]) {
        self.trusted_keys.push(key.to_vec());
    }

    /// Verify a signed manifest
    pub fn verify(&self, manifest: &SignedManifest) -> Result<(), crate::error::Error> {
        // First verify signature is cryptographically valid
        manifest.verify()?;

        // Then verify public key is in trusted list
        if !self.trusted_keys.iter().any(|k| k == &manifest.public_key) {
            return Err(crate::error::Error::PermissionDenied(
                "Manifest signed by untrusted public key".into(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let signer = ManifestSigner::generate();
        let content = b"test manifest content";

        let manifest = SignedManifest::new(content, &signer);

        // Verify self-signed manifest
        assert!(manifest.verify().is_ok());

        // Verify with verifier
        let mut verifier = ManifestVerifier::new(Vec::new());
        verifier.add_trusted_key(&signer.public_key());

        assert!(verifier.verify(&manifest).is_ok());
    }

    #[test]
    fn test_tampered_manifest() {
        let signer = ManifestSigner::generate();
        let mut manifest = SignedManifest::new(b"test", &signer);

        // Tamper with content
        manifest.content[0] ^= 0x01;

        assert!(manifest.verify().is_err());
    }

    #[test]
    fn test_untrusted_key() {
        let signer1 = ManifestSigner::generate();
        let signer2 = ManifestSigner::generate();

        let manifest = SignedManifest::new(b"test", &signer1);

        let verifier = ManifestVerifier::new(vec![signer2.public_key()]);

        assert!(verifier.verify(&manifest).is_err());
    }
}
