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
use thiserror::Error;
use actix_web::http::header::ContentType;

use std::sync::{Arc, Mutex};
use actix_web::{};
use serde::{Serialize, Deserialize};

use crate::repository::db::DbHandle;
use crate::repository::db::EmptyTableError;
use crate::repository::db::QuerryError;
use crate::model::dna_sequence::DnaSequence;


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
    dna_sequences_str: Vec<String>,
}

#[derive(Deserialize)]
pub struct SubmitDnaSequence {
    id: String,
    dna_sequence: String,
}


#[actix_web::get("/get_dna_sequences")]
async fn get_dna_sequences(db: web::Data<Arc<Mutex<DbHandle>>>) -> Result<Json<GetDnaSequencesResponse>, DbDnaSequenceError>{ 
    let db = db.lock().unwrap();
    match db.get_dna_sequences() {
        Ok(a) => { 
            let dna_sequences_str = a.iter().map(|dna_sequence| {
                dna_sequence.to_string()
            }).collect();
            Ok(Json(
                GetDnaSequencesResponse {
                    dna_sequences_str,
                }
            ))
        },
        Err(e) => Err(DbDnaSequenceError::DnaSequenceNotFound(e)),
        //Err(QuerryError::RusqliteError(e)) => panic!("Failed with {}", e),
    }
}


//#[post("/add_item")]
//pub async fn post_item(db: web::Data<Arc<Mutex<DbHandle>>>, 
//        request: Json<SubmitItem>,
//        //name: web::Path<String>, 
//        ) -> Result<Json<String>, DbItemError> {
//    println!("price: {}", &request.price);
//    let price = request.price.parse::<f64>().unwrap();
//    let item = Item::new(request.name.clone(), price);
//    match db.lock().unwrap().push_item(item.id, item.name, item.price) {
//        Ok(id) => Ok(Json(id)),
//        Err(e) => Err(DbItemError::PushFailed(QuerryError::RusqliteError(e))),
//
#[actix_web::post("/create_dna_sequence")]
async fn create_dna_sequence(db: web::Data<Arc<Mutex<DbHandle>>>,
        request: Json<SubmitDnaSequence>,
        ) -> Result<Json<String>, DbDnaSequenceError> {
    let db = db.lock().unwrap();
    let dna_sequence_raw = request.dna_sequence.clone();
    let id = request.id.clone();
    let dna_sequence = DnaSequence::new(id, dna_sequence_raw);
    let reply_id = dna_sequence.id.clone();
    match db.push_dna_sequence(dna_sequence.id) {
        Ok(id) => {
            println!("Inserted dna_sequence {}", id);
            Ok(Json(reply_id))
        },
        Err(e) => Err(DbDnaSequenceError::PushFailed(QuerryError::RusqliteError(e))),
    }
}

