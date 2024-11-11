use reqwest::Client;
use reqwest::Response;
use std::collections::HashMap;

use base64::{Engine as _, engine::general_purpose};


pub fn encode(value: Vec<u8>) -> String {
    general_purpose::STANDARD.encode(value).to_string()
}

pub async fn post_public_key(ip: &str, public_key: Vec<u8>) -> Result<Response, String> {
    let address = ip.to_string() + "/insert_public_key";
    let client = Client::new();
    let mut data = HashMap::new();
    let pk = encode(public_key);
    println!("public key: {}", pk);
    data.insert("id", "".to_string());
    data.insert("public_key", pk);

    let response = match client.post(address)
        .json(&data)
        .send()
        .await {
            Ok(r) => r,
            Err(e) => panic!("Dna post request has failed with: {:?}", e),
        };
    Ok(response)
}

pub async fn post_dna_sequence(ip: &str, id: String, dna_sequence: String, signature: Vec<u8>) -> Result<Response, String> {
    let address = ip.to_string() + "/insert_dna_sequence";
    let client = Client::new();
    let mut data = HashMap::new();
    data.insert("id", id); 
    data.insert("dna_sequence", dna_sequence);
    data.insert("signature", encode(signature));

    let response = match client.post(address)
        .json(&data)
        .send()
        .await {
            Ok(r) => r,
            Err(e) => panic!("Dna post request has failed with: {:?}", e),
        };
    Ok(response)
}

pub async fn get_dna_sequence(ip: &str, id: String) -> Result<Response, String> {
    let address = ip.to_string() + "/dna";
    let client = Client::new();
    let mut data = HashMap::new();
    data.insert("id", id);

    let response = match client.get(address)
        .json(&data)
        .send()
        .await {
            Ok(n) => n,
            Err(e) => panic!("Unable to get html content: {}", e), 
        };
    Ok(response)
}


    
    

