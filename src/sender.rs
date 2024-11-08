pub mod sender{

    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use reqwest::Client;
    use reqwest::Response;
    use tokio::time::{sleep, Duration};
    use crate::model::dna_sequence::DnaSequence;
    use crate::model::public_key::PublicKey;
    use crate::api::dna_sequence::Patch;


    const N_PEERS: u32 = 5;
    const BYZANTINE_THRESHOLD: u32 = (N_PEERS * 2 / 3) + 1; //4. Needs 7 reps to tolerate 2 traitors

    const URL_BASE: &str = "http://";



    pub async fn post_public_key(ip: String, public_key: PublicKey, n_responses: Arc<Mutex<u32>>) -> Result<Response, String> {
        let base = URL_BASE.to_string() + ip.as_ref();
        let address = base + "/share_public_key";
        let client = Client::new();
        let mut data = HashMap::new();
        data.insert("id", public_key.id.clone());
        data.insert("public_key_txt", public_key.encode());
        let response = match client.post(address)
            .json(&data)
            .send()
            .await {
                Ok(r) => r,
                Err(e) => panic!("Item post request failed with {:?}", e),
            };
        *n_responses.lock().unwrap() += 1;
        Ok(response)
    }

    pub async fn post_patch(ip: String, patch: Patch, n_responses: Arc<Mutex<u32>>) -> Result<Response, String> {
        let base = URL_BASE.to_string() + ip.as_ref();
        let address = base + "/share_patch";
        let client = Client::new();
        let mut data = HashMap::new();
        data.insert("id", patch.id);
        data.insert("patch_txt", patch.patch_txt);
        let response = match client.post(address)
            .json(&data)
            .send()
            .await {
                Ok(r) => r,
                Err(e) => panic!("Item post request failed with {:?}", e),
            };
        *n_responses.lock().unwrap() += 1;
        Ok(response)
    }

    pub async fn post_dna_sequence(ip: String, dna_sequence: DnaSequence, n_responses: Arc<Mutex<u32>>) -> Result<Response, String> {
        let base = URL_BASE.to_string() + ip.as_ref();
        let address = base + "/share_dna_sequence";
        println!("posting dna sequence to {}", &address);
        let mut data = HashMap::new();
        data.insert("id", dna_sequence.id);
        data.insert("dna_sequence", dna_sequence.dna_sequence);
        let client = Client::new();

        let response = match client.post(address)
            .json(&data)
            .send()
            .await {
                Ok(r) => r,
                Err(e) => panic!("Item post request failed with {:?}", e),
            };
        *n_responses.lock().unwrap() += 1;
        Ok(response)
    }

    pub async fn broadcast_public_key(addresses: Vec<String>, public_key: PublicKey) {
        let n_responses_arc = Arc::new(Mutex::new(0));
        let n_responses = n_responses_arc.clone();
        for address in addresses {
            let public_key_clone = public_key.clone();//TODO: Expensive clone god knows I tried to avoid
            let address_clone = address.clone();
            let n_responses_clone = n_responses_arc.clone();
            let _ =  tokio::spawn(async move {
                let _ = post_public_key(address_clone, public_key_clone, n_responses_clone).await;
            }).await;
        };

        let mut quorum = false; //this is to minimize the lock usage
        while !quorum {
            tokio::time::sleep(Duration::from_millis(50)).await; //TODO: optimize. Wait/Notify?
            if *n_responses.lock().unwrap() <= BYZANTINE_THRESHOLD {
                quorum = true;
            }
        }
    }

    //TODO: Generic version of theses methods. Data could be a box.
    pub async fn broadcast_patch(addresses: Vec<String>, patch: Patch) {
        let n_responses_arc = Arc::new(Mutex::new(0));
        let n_responses = n_responses_arc.clone();
        for address in addresses {
            let patch_clone = patch.clone();//TODO: Expensive clone god knows I tried to avoid
            let address_clone = address.clone();
            let n_responses_clone = n_responses_arc.clone();
            let _ =  tokio::spawn(async move {
                let _ = post_patch(address_clone, patch_clone, n_responses_clone).await;
            }).await;
        };

        let mut quorum = false; //this is to minimize the lock usage
        while !quorum {
            tokio::time::sleep(Duration::from_millis(50)).await; //TODO: optimize. Wait/Notify?
            if *n_responses.lock().unwrap() <= BYZANTINE_THRESHOLD {
                quorum = true;
            }
        }
    }
    //TODO: Generic version of theses methods. Data could be a box.
    pub async fn broadcast_dna_sequence(addresses: Vec<String>, dna_sequence: DnaSequence) {
        let n_responses_arc = Arc::new(Mutex::new(0));
        let n_responses = n_responses_arc.clone();
        //let lock = Arc::try_unwrap(addresses).unwrap();
        //let addresses = lock.into_inner().unwrap(); 
        for address in addresses {
            let dna_sequence_clone = dna_sequence.clone();//TODO: Expensive clone god knows I tried to avoid
            let address_clone = address.clone();
            let n_responses_clone = n_responses_arc.clone();
            let _ =  tokio::spawn(async move {
                let _ = post_dna_sequence(address_clone, dna_sequence_clone, n_responses_clone).await;
            }).await;
        };

        let mut quorum = false; //this is to minimize the lock usage
        while !quorum {
            tokio::time::sleep(Duration::from_millis(500)).await; //TODO: optimize. Wait/Notify?
            if *n_responses.lock().unwrap() <= BYZANTINE_THRESHOLD {
                quorum = true;
            }
        }
    }


}
    

    
    

