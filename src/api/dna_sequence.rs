use actix_web::{
    get,
    post,
    put,
    error::ResponseError,
    web::Path,
    web::Json,
    web::Data,
    HttpResponse,
    Responder,
    web,
    //http::{header::ContextType, StatusCode}
};
use std::sync::{Arc, Mutex};

use diff_match_patch_rs::{DiffMatchPatch, Efficient, Error, PatchInput};

use thiserror::Error;
use actix_web::http::header::ContentType;
use serde::{Serialize, Deserialize};

use crate::repository::db::DbHandle;
use crate::repository::db::QuerryError;
use crate::model::dna_sequence::DnaSequence;
use crate::sender::sender;
use crate::node::node::Node;
use crate::responses::responses::Responses;


#[derive(Debug, Error, derive_more::Display)]
pub enum DbDnaSequenceError {
    DnaSequenceNotFound(QuerryError),
    PushFailed(QuerryError),
}

impl ResponseError for DbDnaSequenceError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
        .insert_header(ContentType::json())
        .body(self.to_string())
    }
}


#[derive(Serialize)]
struct CreateDnaSequenceResponse {
    id: String,
}

#[derive(Serialize)]
struct GetDnaSequencesResponse { 
    dna_sequence: String,
}

#[derive(Deserialize)]
struct SubmitDiffs {
    id: String,
    diffs: Vec<Diff>,
}

#[derive(Deserialize)]
struct Diff {
    tag: u16,
    index: usize,
    character: char,
}

#[derive(Deserialize)]
pub struct SubmitDnaSequence {
    id: String,
    dna_sequence: String,
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

#[actix_web::get("/get_dna_sequence")]
async fn get_dna_sequence(db: web::Data<Arc<Mutex<DbHandle>>>, user_id: web::Query<UserId>) -> Result<Json<GetDnaSequencesResponse>, DbDnaSequenceError>{ 
    let id = user_id.id.clone();
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
    let (patched_sequence, _) = dmp.patch_apply( //TODO: use ops to check for errors
        &patches, 
        dna_sequence.dna_sequence.as_ref()
    ).unwrap();
    let reply_id = dna_sequence.id.clone();
    let new_sequence = DnaSequence {
        id: reply_id.clone(),
        dna_sequence: patched_sequence,
    };
    match db.push_dna_sequence(&new_sequence) {
        Ok(id) => {
            println!("Inserted dna_sequence {}", id);
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
    let dna_sequence = DnaSequence::new(id, dna_sequence_raw);
    let reply_id = dna_sequence.id.clone();
    match db.push_dna_sequence(&dna_sequence) {
        Ok(id) => {
            println!("Inserted dna_sequence {}", id);
            Ok(Json(reply_id))
        },
        Err(e) => Err(DbDnaSequenceError::PushFailed(QuerryError::RusqliteError(e))),
    }
}

#[actix_web::post("/insert_dna_sequence")]
async fn create_dna_sequence(db: web::Data<Arc<Mutex<DbHandle>>>,
        addresses: web::Data<Arc<Mutex<Vec<String>>>>,
        request: Json<SubmitDnaSequence>,
        ) -> Result<Json<String>, DbDnaSequenceError> {
    let dna_sequence_raw = request.dna_sequence.clone();
    let id = request.id.clone();
    let dna_sequence = DnaSequence::new(id, dna_sequence_raw);
    let reply_id = dna_sequence.id.clone();
    let db = db.lock().unwrap();
    //if the sequence already exists, broadcast patches
    let seq = match db.get_dna_sequence(request.id.clone()) {
        Ok(old_seq) => {
            let dmp = DiffMatchPatch::new();
            let diffs = dmp.diff_main::<Efficient>(old_seq.dna_sequence.as_ref(), 
                request.dna_sequence.as_ref())
                .unwrap();
            let patches = dmp.patch_make(PatchInput::new_diffs(&diffs)).unwrap();
            let patch_txt = dmp.patch_to_text(&patches);
            match db.push_dna_sequence(&dna_sequence) {
                Ok(id) => {
                    println!("Inserted dna_sequence {}", id);
                    let patch = Patch {
                        id,
                        patch_txt,
                    };

                    let _ = tokio::spawn(async move {
                        sender::broadcast_patch(addresses.as_ref().to_owned(), patch).await; //TODO: this needs to have transaction semantics
                    }).await;
                },
                Err(e) => {
                    return Err(DbDnaSequenceError::PushFailed(QuerryError::RusqliteError(e)));
                },
            }
            Some(old_seq)
        },
        Err(_) =>  {
            match db.push_dna_sequence(&dna_sequence) {
                Ok(id) => {
                    println!("Inserted dna_sequence {}", id);
                    let _ = tokio::spawn(async move {
                        sender::broadcast_dna_sequence(addresses.as_ref().to_owned(), dna_sequence).await; //TODO: this needs to have transaction semantics
                    }).await;
                },
                Err(e) => {
                    return Err(DbDnaSequenceError::PushFailed(QuerryError::RusqliteError(e)));
                },
            }
            None
        }
    };
    Ok(Json(reply_id))
}

