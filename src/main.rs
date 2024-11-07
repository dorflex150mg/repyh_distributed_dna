pub mod node;
pub mod user;
pub mod server;
pub mod sender;
pub mod responses;

use crate::node::node::Node;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::env;


use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use actix_web::{web, App, HttpServer};
use serde::{Serialize, Deserialize};

use crate::repository::db::DbHandle;
use crate::responses::responses::Responses;
use tracing::{debug, info};

mod api;
mod repository;
mod model;


use api::dna_sequence::{
    dna,
    insert_dna_sequence,
    share_patch,
    share_dna_sequence
    
};


type Db = Arc<Mutex<DbHandle>>;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //init_tracing();
    println!("Starting");
    //Loading conf files with peer ips
    let file_name = env::var("FILENAME").unwrap();
    println!("filename: {}", &file_name);
    let mut file = File::open(file_name).unwrap();
    let mut ips_str = String::new();
    file.read_to_string(&mut ips_str).unwrap();
    let json: Vec<Vec<String>> = serde_json::from_str(ips_str.as_ref()).unwrap();
    let ip_list = json[0].clone();
    let ip = ip_list[0].clone();
    let api_ip = ip_list[1].clone();
    let peers = json[1].clone();
    // Initializing a node
    let responses: HashMap<String, Responses> = HashMap::new();
    //let addresses_arc = Arc::new(Mutex::new(peers));
    let responses_arc = Arc::new(Mutex::new(responses));
    //Creating client-side service
    let db_name = env::var("DATABASE").unwrap();
    println!("database: {}", db_name);
    let db: Db = Arc::new(Mutex::new(DbHandle::new(String::from(db_name)).unwrap()));
    println!("Listening on: {}", &api_ip);
    let _ = HttpServer::new(move || { 
        let db_handle = web::Data::new(db.clone()); //a struct that represents data
        let addresses_data = web::Data::new(peers.clone()); 
        let responses_data = web::Data::new(responses_arc.clone());
        println!("Creating app...");
        App::new()
            .service(insert_dna_sequence)
            .service(dna)
            .service(share_patch)
            .service(share_dna_sequence)
            .app_data(addresses_data)
            .app_data(responses_data)
            .app_data(db_handle) //enrolls data "type" into the app
    })
        .bind(api_ip)?
        .run()
        .await;
    Ok(())
}


//pub fn init_tracing() {
//    use tracing::level_filters::LevelFilter;
//    use tracing_subscriber::prelude::*;
//    use tracing_subscriber::EnvFilter;
//
//    let env = EnvFilter::builder()
//        .with_default_directive(LevelFilter::DEBUG.into())
//        .with_env_var("RUST_LOG")
//        .from_env_lossy();
//
//    let fmt_layer = tracing_subscriber::fmt::layer()
//        .compact()
//        .with_file(true)
//        .with_line_number(true)
//        .with_thread_ids(false)
//        .with_target(false);
//    tracing_subscriber::registry()
//        .with(fmt_layer)
//        .with(env)
//        .init();
//}
