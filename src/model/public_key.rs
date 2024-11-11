use std::fmt::{self, Display};
use std::sync::Arc;
use uuid::Uuid;
use tracing::{debug, info};
use serde::{Serialize, Deserialize};
use base64::{Engine as _, engine::general_purpose};
use thiserror::Error;
use ring::signature::{self, UnparsedPublicKey};

/// Structure representing a public key with an optional encoded key.
#[derive(Serialize, Deserialize, Clone)]
pub struct PublicKey {
    pub id: Arc<str>, // Unique identifier for the public key.
    pub public_key: Option<Vec<u8>>, // Encoded public key data.
}

/// Error types for failures in base64 decoding of public keys.
#[derive(Error, Debug, derive_more::From, derive_more::Display)]
pub enum PublicKeyFromBase64Error {
    Base64Error(base64::DecodeError),
}

/// Error types for signature verification failures.
#[derive(Error, Debug, derive_more::From)]
pub enum WrongSignatureError {
    #[error("Verification of signature failed -- Bad Signature")]
    VerificationFailed,
    #[error("The key provided is not present in the database.")]
    NoPublicKey,
}

impl PublicKey {
    /// Creates a new public key with a generated UUID and given key data.
    pub fn new(public_key: Vec<u8>) -> Self {
        let id = Uuid::new_v4().to_string();
        PublicKey {
            id: id.into(),
            public_key: Some(public_key),
        }
    }

    /// Constructs a public key from a raw base64-encoded string and an ID.
    pub fn from_raw(id: String, string: String) -> Result<Self, PublicKeyFromBase64Error> {
        Ok(PublicKey {
            id: id.into(),
            public_key: Some(general_purpose::STANDARD.decode(string)?),
        })
    }

    /// Encodes the public key to a base64 string.
    pub fn encode(self) -> String {
        let pk: &Vec<u8> = &self.public_key.unwrap();
        general_purpose::STANDARD.encode(pk).to_string()
    }

    /// Verifies a message's signature with the provided public key.
    pub fn check_signature(signature: Arc<str>, public_key: PublicKey, message: Arc<str>) -> Result<(), WrongSignatureError> {
        match public_key.public_key {
            Some(pk) => {
                let raw_signature = general_purpose::STANDARD.decode(signature.to_string()).unwrap();
                let peer_public_key = UnparsedPublicKey::new(&signature::ED25519, pk);
                match peer_public_key.verify(&message.as_bytes(), &raw_signature.as_ref()) {
                    Ok(()) => Ok(()),
                    //Err(_) => Ok(()), // TODO: revisit this for proper error handling.
                    Err(e) => {
                        debug!("VERIFICATION FAILED!: {}", e);
                        Err(WrongSignatureError::VerificationFailed) 
                    },
                }
            },
            None => Err(WrongSignatureError::NoPublicKey),
        }
    }
}

impl TryFrom<String> for PublicKey {
    type Error = PublicKeyFromBase64Error;
    /// Attempts to create a `PublicKey` from a base64-encoded string.
    fn try_from(string: String) -> Result<Self, Self::Error> {
        Ok(PublicKey {
            id: Uuid::new_v4().to_string().into(),
            public_key: Some(general_purpose::STANDARD.decode(string)?),
        })
    }
}

impl Display for PublicKey {
    /// Formats the public key for display using its ID.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

