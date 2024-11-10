use actix_web::{
    error::ResponseError,
    web::Json,
    HttpResponse,
    web,
};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use actix_web::http::header::ContentType;
use serde::Deserialize;
use crate::repository::db::DbHandle;
use crate::repository::db::QuerryError;
use crate::model::public_key::PublicKey;
use crate::sender::sender;
use tracing::{debug, info};

/// Errors for public key operations.
#[derive(Debug, Error, derive_more::Display)]
pub enum DbPublicKeyError {
    PushFailed(QuerryError),
}

impl ResponseError for DbPublicKeyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }
}

/// Request structure for submitting a new public key.
#[derive(Deserialize)]
pub struct SubmitPublicKey {
    id: String,
    public_key: String,
}

/// Handler for receiving a public key broadcast.
#[actix_web::post("/share_public_key")]
async fn share_public_key(
    db: web::Data<Arc<Mutex<DbHandle>>>,
    request: Json<SubmitPublicKey>,
) -> Result<Json<String>, DbPublicKeyError> {
    let db = db.lock().unwrap();
    let public_key_encoded = request.public_key.clone();
    let id = request.id.clone();
    let public_key = PublicKey::from_raw(id, public_key_encoded).unwrap();
    match db.push_public_key(&public_key) {
        Ok(id) => Ok(Json(id.to_string())),
        Err(e) => Err(DbPublicKeyError::PushFailed(QuerryError::RusqliteError(e))),
    }
}

/// Handler for inserting a new public key and broadcasting it.
#[actix_web::post("/insert_public_key")]
async fn insert_public_key(
    db: web::Data<Arc<Mutex<DbHandle>>>,
    addresses: web::Data<Vec<String>>,
    request: Json<SubmitPublicKey>,
) -> Result<Json<String>, DbPublicKeyError> {
    debug!("Creating public key");
    let public_key_encoded = request.public_key.clone();
    let public_key = PublicKey::try_from(public_key_encoded).unwrap();
    let id = public_key.id.clone();
    debug!("locking db");
    let db = db.lock().unwrap();
    match db.push_public_key(&public_key) {
        Ok(_) => {
            debug!("inserting new pk");
            let _ = tokio::spawn(async move {
                sender::broadcast_public_key(addresses.as_ref().to_owned(), public_key).await;
            }).await;
        },
        Err(e) => return Err(DbPublicKeyError::PushFailed(QuerryError::RusqliteError(e))),
    };
    Ok(Json(id.to_string()))
}

