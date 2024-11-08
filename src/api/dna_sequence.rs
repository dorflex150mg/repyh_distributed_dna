use actix_web::{
    error::ResponseError,
    web::Json,
    HttpResponse,
    web,
};
use std::sync::{Arc, Mutex};

use diff_match_patch_rs::{DiffMatchPatch, Efficient, PatchInput};

use thiserror::Error;
use actix_web::http::header::ContentType;
use serde::{Serialize, Deserialize};

use crate::repository::db::DbHandle;
use crate::repository::db::QuerryError;
use crate::model::dna_sequence::DnaSequence;
use crate::model::public_key::PublicKey;
use crate::model::public_key::WrongSignatureError;
use crate::sender::sender;


use tracing::{debug, info};


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


#[derive(Serialize)]
struct GetDnaSequencesResponse { 
    dna_sequence: String,
}

#[derive(Deserialize)]
pub struct SubmitDnaSequence {
    id: String,
    dna_sequence: String,
    signature: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Patch {
    pub id: String,
    pub patch_txt: String,
}

#[derive(Deserialize)]
pub struct UserId {
    id: String,
}

#[actix_web::get("/dna")]
async fn dna(db: web::Data<Arc<Mutex<DbHandle>>>, 
    request: Json<UserId>,
    ) -> Result<Json<GetDnaSequencesResponse>, DbDnaSequenceError>{ 
    let id = request.id.clone();
    let db = db.lock().unwrap();
    match db.get_dna_sequence(id) {
        Ok(read_seq) => { 
            Ok(Json(
                GetDnaSequencesResponse {
                    dna_sequence: read_seq.dna_sequence,
                }
            ))
        },
        Err(e) => Err(DbDnaSequenceError::DnaSequenceNotFound(e)),
    }
}

#[actix_web::post("/share_patch")]
async fn share_patch(db: web::Data<Arc<Mutex<DbHandle>>>,
        request: Json<Patch>,
        ) -> Result<Json<String>, DbDnaSequenceError> {
    let patch = request.patch_txt.clone();
    let id = request.id.clone();
    let db = db.lock().unwrap();
    let dna_sequence = db.get_dna_sequence(id).expect("Error -- No dna sequence with given id");

    let dmp = DiffMatchPatch::new();
    let patches = dmp.patch_from_text::<Efficient>(patch.as_ref()).unwrap();
    let (patched_sequence, ops) = dmp.patch_apply( 
        &patches, 
        dna_sequence.dna_sequence.as_ref()
    ).unwrap();
    let mut success = true;
    ops.iter().for_each(|&o| success = success && o);
    if !success {
        return Err(DbDnaSequenceError::PatchFailed);
    }

    let reply_id = dna_sequence.id.clone();
    let new_sequence = DnaSequence {
        id: reply_id.clone(),
        dna_sequence: patched_sequence,
    };
    match db.push_dna_sequence(&new_sequence) {
        Ok(id) => {
            Ok(Json(reply_id))
        },
        Err(e) => Err(DbDnaSequenceError::PushFailed(QuerryError::RusqliteError(e))),
    }
}

#[actix_web::post("/share_dna_sequence")]
async fn share_dna_sequence(db: web::Data<Arc<Mutex<DbHandle>>>,
        request: Json<SubmitDnaSequence>,
        ) -> Result<Json<String>, DbDnaSequenceError> {
    let db = db.lock().unwrap();
    let dna_sequence_raw = request.dna_sequence.clone();
    let id = request.id.clone();
    let mut dna_sequence = DnaSequence::new(id, dna_sequence_raw);
    dna_sequence.id = request.id.clone();
    let reply_id = dna_sequence.id.clone();
    match db.push_dna_sequence(&dna_sequence) {
        Ok(id) => {
            Ok(Json(reply_id))
        },
        Err(e) => Err(DbDnaSequenceError::PushFailed(QuerryError::RusqliteError(e))),
    }
}

#[actix_web::post("/insert_dna_sequence")]
async fn insert_dna_sequence(db: web::Data<Arc<Mutex<DbHandle>>>,
        addresses: web::Data<Vec<String>>,
        request: Json<SubmitDnaSequence>,
        ) -> Result<Json<String>, DbDnaSequenceError> {
    debug!("Creating dna sequence");
    let dna_sequence_raw = request.dna_sequence.clone();
    let id = request.id.clone();
    let mut dna_sequence = DnaSequence::new(id, dna_sequence_raw);
    let reply_id = dna_sequence.id.clone();
    debug!("locking db");
    let db = db.lock().unwrap();
    let public_key = db.get_public_key(&reply_id).unwrap();
    PublicKey::check_signature(&request.signature.clone(), public_key, &dna_sequence.dna_sequence)
        .map_err(DbDnaSequenceError::SignatureVerificationFailed)?;
    //if the sequence already exists, broadcast patches
    let _ = match db.get_dna_sequence(request.id.clone()) {
        Ok(old_sequence) => { 
            dna_sequence.id = old_sequence.id.clone(); //TODO: dna_sequence builder: with_id()
            let dmp = DiffMatchPatch::new();
            let diffs = dmp.diff_main::<Efficient>(
                old_sequence.dna_sequence.as_ref(), 
                request.dna_sequence.as_ref()
            ).unwrap();
            let patches = dmp.patch_make(PatchInput::new_diffs(&diffs)).unwrap();
            let patch_txt = dmp.patch_to_text(&patches);
            match db.push_dna_sequence(&dna_sequence) {
                Ok(id) => {
                    let patch = Patch {id, patch_txt};
                    let _ = tokio::spawn(async move {
                        //TODO: this needs to have transaction semantics
                        sender::broadcast_patch(addresses.as_ref().to_owned(), patch).await; 
                    }).await;
                },
                Err(e) => return Err(DbDnaSequenceError::PushFailed(QuerryError::RusqliteError(e))),
            }
            Some(old_sequence)
        },
        Err(_) =>  {
            match db.push_dna_sequence(&dna_sequence) {
                Ok(_) => {
                    debug!("inserting new sequence");
                    let _ = tokio::spawn(async move {
                        sender::broadcast_dna_sequence(addresses.as_ref().to_owned(), dna_sequence).await; 
                    }).await;
                },
                Err(e) => return Err(DbDnaSequenceError::PushFailed(QuerryError::RusqliteError(e))),
            }
            None
        }
    };
    Ok(Json(reply_id))
}

