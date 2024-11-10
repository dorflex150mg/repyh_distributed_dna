mod dna_client;
mod client_sender;

use crate::dna_client::dna_client::DnaClient;
use tracing::{debug, info};

const IP: &str = "http://127.0.0.1:8082";
const ANOTHER_IP: &str = "http://127.0.0.1:8083";

#[tokio::main]
async fn main() {
    init_tracing();

    let mut dna_client = DnaClient::new("TACG".to_string());
    let pk_response = client_sender::post_public_key(IP, dna_client.get_pub_key()).await.unwrap();
    info!("Public key post response: {:?}", &pk_response);
    
    let id = pk_response
        .text()
        .await
        .unwrap()
        .trim_matches('\"')
        .to_string();
    info!("id: {}", id);
    let signature = dna_client.sign();

    let dna_response = client_sender::post_dna_sequence(
        IP, 
        id.clone(), 
        dna_client.dna_sequence.clone(), 
        signature.clone()
    ).await.unwrap();
    info!("Dna sequence post response: {:?}", dna_response);

    let dna_get_response = client_sender::get_dna_sequence(IP, id.clone()).await.unwrap(); 
    info!("Dna sequence get response: {:?}", dna_get_response);

    dna_client.set_dna_sequence("TCCG");
    let patch_response = client_sender::post_dna_sequence(IP, id.clone(), dna_client.dna_sequence.clone(), signature).await.unwrap();
    info!("Dna patch post response: {:?}", patch_response);

    let dna_get_response = client_sender::get_dna_sequence(IP, id.clone()).await.unwrap(); 
    info!("Dna patch get response: {:?}", dna_get_response);
    
    let dna_get_response = client_sender::get_dna_sequence(ANOTHER_IP, id).await.unwrap(); 
    info!("Dna patch get response: {:?}", &dna_get_response);
    info!("Dna patch response: {}", dna_get_response.text().await.unwrap().trim_matches('\"').to_string());

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
}

    
