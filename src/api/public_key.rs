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
use crate::model::public_key::PublicKey;
use crate::sender::sender;
use crate::node::node::Node;

use tracing::{debug, info};


#[derive(Debug, Error, derive_more::Display)]
pub enum DbPublicKeyError {
    PublicKeyNotFound(QuerryError),
    PushFailed(QuerryError),
}

impl ResponseError for DbPublicKeyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
        .insert_header(ContentType::json())
        .body(self.to_string())
    }
}

#[derive(Serialize)]
struct CreatePublicKeyResponse {
    id: String,
}

#[derive(Deserialize)]
pub struct SubmitPublicKey {
    id: String,
    public_key: String,
}


#[derive(Deserialize)]
pub struct UserId {
    id: String,
}

#[actix_web::post("/share_public_key")]
async fn share_public_key(db: web::Data<Arc<Mutex<DbHandle>>>,
        request: Json<SubmitPublicKey>,
        ) -> Result<Json<String>, DbPublicKeyError> {
    println!("Received public key");
    let db = db.lock().unwrap();
    let public_key_encoded = request.public_key.clone();
    let id = request.id.clone();
    let reply_id = id.clone();
    let public_key = PublicKey::from_raw(id, public_key_encoded).unwrap();
    match db.push_public_key(&public_key) {
        Ok(id) => {
            println!("Inserted public_key {}", id);
            Ok(Json(reply_id))
        },
        Err(e) => Err(DbPublicKeyError::PushFailed(QuerryError::RusqliteError(e))),
    }
}

#[actix_web::post("/insert_public_key")]
async fn insert_public_key(db: web::Data<Arc<Mutex<DbHandle>>>,
        addresses: web::Data<Vec<String>>,
        request: Json<SubmitPublicKey>,
        ) -> Result<Json<String>, DbPublicKeyError> {
    debug!("Creating public key");
    let public_key_encoded = request.public_key.clone();
    let id = request.id.clone();
    let reply_id = id.clone();
    let public_key = PublicKey::from_raw(id, public_key_encoded).unwrap();
    debug!("locking db");
    let db = db.lock().unwrap();
    //if the sequence already exists, broadcast patches
    match db.push_public_key(&public_key) {
        Ok(_) => {
            debug!("inserting new pk");
            let _ = tokio::spawn(async move {
                sender::broadcast_public_key(addresses.as_ref().to_owned(), public_key).await; 
            }).await;
        },
        Err(e) => return Err(DbPublicKeyError::PushFailed(QuerryError::RusqliteError(e))),
    };
    Ok(Json(reply_id))
}

