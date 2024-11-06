pub mod node;
pub mod user;
pub mod server;

use crate::node::node::Node;
use crate::user::user::User;
use std::error::Error;
use std::fs::File;
use std::io::Read;


use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_rt;
use serde::{Serialize, Deserialize};

use crate::repository::db::DbHandle;
//use crate::api::item::get_item;
use tracing::{debug, info};

mod api;
mod repository;
mod model;


use api::dna_sequence::{
    create_dna_sequence,
    get_dna_sequences,
};


type Db = Arc<Mutex<DbHandle>>;

//#[actix_web::get("/hello/{id}")]
//async fn hello(user_id: web::Path<u64>) -> impl Responder { //formats from endpoint into Responder
//    format!("Hello, {user_id}!")
//}

#[tokio::main]
async fn  main() -> Result<(), Box<dyn Error>> {
    //Loading conf files with peer ips
    let mut file = File::open("conf/ips0.json").unwrap();
    let mut ips_str = String::new();
    file.read_to_string(&mut ips_str).unwrap();
    let json: Vec<Vec<String>> = serde_json::from_str(ips_str.as_ref()).unwrap();
    let ip_list = json[0].clone();
    let ip = ip_list[0].clone();
    let api_ip = ip_list[1].clone();
    let peers = json[1].clone();
    // Initializing a node
    tokio::spawn(async move {
        let node = Node::new(ip, peers).await.unwrap();
        let user = User::new();
        info!("server id: {}, node id: {}", node.server.id, node.server.id);
        info!("user id: {}", user.id);
    });

    // Creating client-side service
    let db: Db = Arc::new(Mutex::new(DbHandle::new(String::from("dna.db")).unwrap()));
    let _ = HttpServer::new(move || { 
        let db_handle = web::Data::new(db.clone()); //a struct that represents data
        App::new()
            .service(get_dna_sequences)
            .app_data(db_handle) //enrolls data "type" into the app
    })
        .bind(api_ip)?
        .run()
        .await;
    init_tracing();
    loop {}
    Ok(())
}


pub fn init_tracing() {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;

    let env = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .with_env_var("RUST_LOG")
        .from_env_lossy();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(false)
        .with_target(false);
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env)
        .init();
}
