use crate::{
    model::{
        public_key::{PublicKey, WrongSignatureError},
        dna_sequence::DnaSequence,
        patch::Patch,
    },
    repository::db::{DbHandle, QuerryError},
    sender,
};

use std::sync::{Arc, Mutex};

use serde::{Serialize, Deserialize};
use tracing::{debug, info};
use diff_match_patch_rs::{DiffMatchPatch, Efficient, PatchInput};
use thiserror::Error;
use actix_web::{
    http::header::ContentType,
    error::ResponseError,
    web::Json,
    HttpResponse,
    web,
};

/// Errors for DNA sequence operations.
#[derive(Debug, Error, derive_more::Display)]
pub enum DbDnaSequenceError { 
    DnaSequenceNotFound(QuerryError),
    PushFailed(QuerryError),
    SignatureVerificationFailed(WrongSignatureError),
    PatchFailed,
}

impl ResponseError for DbDnaSequenceError { 
    fn error_response(&self) -> HttpResponse { 
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    } 
}

/// Response for retrieving DNA sequences.
#[derive(Serialize)]
struct GetDnaSequencesResponse { 
    dna_sequence: String,
}

/// Request structure for submitting a new DNA sequence.
#[derive(Deserialize)]
pub struct SubmitDnaSequence { 
    id: Arc<str>,
    dna_sequence: Arc<str>,
    signature: Arc<str>,
}

#[derive(Deserialize)] 
pub struct SubmitPatch {
    id: Arc<str>,
    patch_txt: Arc<str>,
    signature: Arc<str>,
}

/// Structure for ID only requests.
#[derive(Deserialize)]
pub struct ClientId { 
    id: Arc<str>,
}

/// Handler for retrieving DNA sequences by ID.
#[actix_web::get("/dna")]
async fn dna(
    db: web::Data<Arc<Mutex<DbHandle>>>, 
    request: Json<ClientId>,
) -> Result<Json<GetDnaSequencesResponse>, DbDnaSequenceError> { 
    let id = request.id.clone();
    let db = db.lock().unwrap();
    match db.get_dna_sequence(id) { 
        Ok(read_seq) => Ok(Json(GetDnaSequencesResponse { dna_sequence: read_seq.dna_sequence.to_string() })),
        Err(e) => Err(DbDnaSequenceError::DnaSequenceNotFound(e)),
    } 
}

/// Handler for shared patches.
#[actix_web::post("/share_patch")]
async fn share_patch(
    db: web::Data<Arc<Mutex<DbHandle>>>,
    request: Json<SubmitPatch>,
) -> Result<Json<String>, DbDnaSequenceError> { 

    let patch = request.patch_txt.clone();
    let id = request.id.clone();
    let signature = request.signature.clone();
    let db = db.lock().unwrap();
    let dna_sequence = db.get_dna_sequence(id.clone()).expect("Error -- No dna sequence with given id");
    let dmp = DiffMatchPatch::new();
    let patches = dmp.patch_from_text::<Efficient>(patch.as_ref()).unwrap();
    let (patched_sequence_str, ops) = dmp.patch_apply(&patches, dna_sequence.dna_sequence.as_ref()).unwrap();
    let patched_sequence: Arc<str> = patched_sequence_str.into();
    let mut success = true;
    ops.iter().for_each(|&o| success = success && o);
    if !success { 
        return Err(DbDnaSequenceError::PatchFailed);
    } 

    //retrieving that id's public key
    let public_key = db.get_public_key(id.clone()).unwrap();

    //checking the signature with that id's public key - we check the patched value.
    PublicKey::check_signature(signature.clone(), public_key, patched_sequence.clone())
        .map_err(DbDnaSequenceError::SignatureVerificationFailed)?;


    let new_sequence = DnaSequence::new(id.clone(), patched_sequence.clone());
    match db.push_dna_sequence(&new_sequence) { 
        Ok(id) => Ok(Json(id.clone().to_string())),
        Err(e) => Err(DbDnaSequenceError::PushFailed(QuerryError::RusqliteError(e))),
    } 
}

/// Handler for shared DNA sequences from another peer.
#[actix_web::post("/share_dna_sequence")]
async fn share_dna_sequence(
    db: web::Data<Arc<Mutex<DbHandle>>>,
    request: Json<SubmitDnaSequence>,
) -> Result<Json<String>, DbDnaSequenceError> { 
    let dna_sequence_raw = request.dna_sequence.clone();
    let id = request.id.clone();
    let signature = request.signature.clone(); 
    let dna_sequence = DnaSequence::new(id.clone(), dna_sequence_raw.clone());

    //retrieving that id's public key
    let db = db.lock().unwrap();
    let public_key = db.get_public_key(id.clone()).unwrap();

    //checking the signature with that id's public key - nodes check signatures of shared dna
    PublicKey::check_signature(signature.clone(), public_key, dna_sequence_raw.clone())
        .map_err(DbDnaSequenceError::SignatureVerificationFailed)?;

    match db.push_dna_sequence(&dna_sequence) { 
        Ok(id) => Ok(Json(id.to_string())),
        Err(e) => Err(DbDnaSequenceError::PushFailed(QuerryError::RusqliteError(e))),
    } 
}

/// Handler for inserting a new DNA sequence and applying patches.
#[actix_web::post("/insert_dna_sequence")]
async fn insert_dna_sequence(
    db: web::Data<Arc<Mutex<DbHandle>>>,
    addresses: web::Data<Vec<String>>,
    request: Json<SubmitDnaSequence>,
) -> Result<Json<String>, DbDnaSequenceError> { 
    let dna_sequence_raw = request.dna_sequence.clone();
    let id = request.id.clone();
    debug!("id: {}", &id);
    let signature = request.signature.clone();
    let mut dna_sequence = DnaSequence::new(id.clone(), dna_sequence_raw.clone());

    //retrieving that id's public key
    let db = db.lock().unwrap();
    let public_key = db.get_public_key(id.clone()).unwrap();

    //checking the signature with that id's public key.
    PublicKey::check_signature(signature.clone(), public_key, dna_sequence_raw.clone())
        .map_err(DbDnaSequenceError::SignatureVerificationFailed)?;

    let _ = match db.get_dna_sequence(request.id.clone()) { 
        Ok(old_sequence) => { 
            debug!("Existing sequence found");
            dna_sequence.id = old_sequence.id.clone();

            // Sending patches
            let dmp = DiffMatchPatch::new();
            let diffs = dmp.diff_main::<Efficient>(
                old_sequence.dna_sequence.as_ref(), 
                request.dna_sequence.as_ref()
            ).unwrap();
            let patches = dmp.patch_make(PatchInput::new_diffs(&diffs)).unwrap();
            let patch_txt: Arc<str> = dmp.patch_to_text(&patches).into();

            let res = db.push_dna_sequence(&dna_sequence); 
            drop(db);
            match res {
                Ok(id) => { 
                    let patch = Patch::new(id, patch_txt);
                    let _ = tokio::spawn(async move { 
                        sender::broadcast_patch(addresses.as_ref().to_owned(), signature, patch).await; 
                    }).await;
                },
                Err(e) => return Err(DbDnaSequenceError::PushFailed(QuerryError::RusqliteError(e))),
            }
            Some(old_sequence)
        },
        Err(_) => { 
            info!("Pushing new sequence");
            match db.push_dna_sequence(&dna_sequence) { 
                Ok(_) => { 
                    debug!("inserting new sequence");
                    let _ = tokio::spawn(async move { 
                        sender::broadcast_dna_sequence(addresses.as_ref().to_owned(), dna_sequence, signature).await; 
                    }).await;
                },
                Err(e) => return Err(DbDnaSequenceError::PushFailed(QuerryError::RusqliteError(e))),
            }
            None
        }
    };
    Ok(Json(id.clone().to_string()))

}

