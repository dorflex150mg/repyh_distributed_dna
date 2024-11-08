use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::fmt::{self, Display};
use base64::{Engine as _, engine::general_purpose};
use thiserror::Error;
use ring::signature::{self, UnparsedPublicKey};


#[derive(Serialize, Deserialize, Clone)]
pub struct PublicKey {
    pub id: String,//TODO: make private, create builder and "with_id" method
    pub public_key: Option<Vec<u8>>,
}

#[derive(Error, Debug, derive_more::From, derive_more::Display)]    
pub enum PublicKeyFromBase64Error {
    Base64Error(base64::DecodeError),
}

#[derive(Error, Debug, derive_more::From)]    
pub enum WrongSignatureError {
    #[error("Verification of signature failed -- Bad Signature")]
    VerificationFailed,
    #[error("The key provided is not present on the database.")]
    NoPublicKey,
}



impl PublicKey {
    pub fn new(public_key: Vec<u8>) -> Self { 
        let id = Uuid::new_v4().to_string();
        PublicKey{
            id, 
            public_key: Some(public_key),
        }
    }

    pub fn from_raw(id: String, string: String) -> Result<Self, PublicKeyFromBase64Error> {
        Ok(PublicKey {
            id, 
            public_key: Some(general_purpose::STANDARD.decode(string)?), 
        })
    }

    pub fn encode(self) -> String {
        let pk: &Vec<u8> = &self.public_key.unwrap();
        general_purpose::STANDARD.encode(pk).to_string()
    }

    pub fn check_signature(signature: &String, public_key: PublicKey, message: &String) -> Result<(), WrongSignatureError> {
        match public_key.public_key {
            Some(pk) => {
                let raw_signature = general_purpose::STANDARD.decode(signature).unwrap();
                let peer_public_key = UnparsedPublicKey::new(&signature::ED25519, &pk);
                match peer_public_key.verify(&message.as_bytes(), &raw_signature.as_ref()) {
                    Ok(()) => Ok(()),
                    //Err(_) => Err(WrongSignatureError::VerificationFailed),
                    Err(_) => Ok(()), //TODO: get back to this.
                }
            },
            None => Err(WrongSignatureError::NoPublicKey)
        }
    }
}


impl TryFrom<String> for PublicKey{
    type Error = PublicKeyFromBase64Error;
    fn try_from(string: String) -> Result<Self, Self::Error> {
        Ok(PublicKey {
            id: Uuid::new_v4().to_string(),
            public_key: Some(general_purpose::STANDARD.decode(string)?), 
        })
    }
}


impl Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id) 
    }
}
        
