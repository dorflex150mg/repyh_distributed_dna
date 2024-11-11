pub mod sender{

    use crate::model::{
        dna_sequence::DnaSequence,
        public_key::PublicKey,
        patch::Patch,
    };
    use std::{
        sync::{Arc, Mutex},
        collections::HashMap,
    };
    use reqwest::{Client, Response};
    use tokio::time::{sleep, Duration};


    const N_PEERS: u32 = 5;
    const BYZANTINE_THRESHOLD: u32 = (N_PEERS * 2 / 3) + 1; //4. Needs 7 reps to tolerate 2 traitors

    const URL_BASE: &str = "http://";


    pub async fn post_public_key(
        ip: String, 
        public_key: PublicKey, 
        n_responses: Arc<Mutex<u32>>
    ) -> Result<Response, String> {
        let base = URL_BASE.to_string() + ip.as_ref();
        let address = base + "/share_public_key";
        let client = Client::new();
        let id = public_key.id.clone();
        let encoded_key: Arc<str> = public_key.encode().into();
        let data = HashMap::from([
            ("id", id),
            ("public_key", encoded_key),
        ]);
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

    pub async fn post_patch(
        ip: String, 
        patch: Patch, 
        signature: Arc<str>,
        n_responses: Arc<Mutex<u32>>
    ) -> Result<Response, String> {
        let base = URL_BASE.to_string() + ip.as_ref();
        let address = base + "/share_patch";
        let client = Client::new();
        let data = HashMap::from([
            ("id", patch.id),
            ("patch_txt", patch.patch_txt),
            ("signature", signature),
        ]);
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

    pub async fn post_dna_sequence(
        ip: String, 
        dna_sequence: DnaSequence, 
        signature: Arc<str>, 
        n_responses: Arc<Mutex<u32>>
    ) -> Result<Response, String> {
        let base = URL_BASE.to_string() + ip.as_ref();
        let address = base + "/share_dna_sequence";
        println!("Posting dna sequence to {} with id {}", ip, dna_sequence.id.clone());
        let data = HashMap::from([
            ("id", dna_sequence.id),
            ("dna_sequence", dna_sequence.dna_sequence),
            ("signature", signature),
        ]);
        let client = Client::new();

        let response = match client.post(address)
            .json(&data)
            .send()
            .await {
                Ok(r) => r,
                Err(e) => panic!("Item post request failed with {:?}", e),
            };
        println!("Response: {:?}", response);
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
    //Would probably require reflection.
    pub async fn broadcast_patch(addresses: Vec<String>, signature: Arc<str>, patch: Patch) {
        let n_responses_arc = Arc::new(Mutex::new(0));
        let n_responses = n_responses_arc.clone();
        for address in addresses {
            let patch_clone = patch.clone();//TODO: Expensive clone god knows I tried to avoid
            let address_clone = address.clone();
            let signature_clone = signature.clone();
            let n_responses_clone = n_responses_arc.clone();
            let _ =  tokio::spawn(async move {
                let _ = post_patch(address_clone, patch_clone, signature_clone, n_responses_clone).await;
            }).await;
        };

        let mut quorum = false; //this is to minimize the lock usage
        while !quorum {
            sleep(Duration::from_millis(50)).await; //TODO: optimize. Wait/Notify?
            if *n_responses.lock().unwrap() <= BYZANTINE_THRESHOLD {
                quorum = true;
            }
        }
    }
    //TODO: Generic version of theses methods. Data could be a box.
    pub async fn broadcast_dna_sequence(addresses: Vec<String>, dna_sequence: DnaSequence, signature: Arc<str>) {
        let n_responses_arc = Arc::new(Mutex::new(0));
        let n_responses = n_responses_arc.clone();
        for address in addresses {
            let dna_sequence_clone = dna_sequence.clone();//TODO: Expensive clone god knows I tried to avoid
            let address_clone = address.clone();
            let n_responses_clone = n_responses_arc.clone();
            let signature = signature.clone();
            let _ =  tokio::spawn(async move {
                let _ = post_dna_sequence(address_clone, dna_sequence_clone, signature, n_responses_clone).await;
            }).await;
        };

        let mut quorum = false; //this is to minimize the lock usage
        while !quorum {
            sleep(Duration::from_millis(500)).await; //TODO: optimize. Wait/Notify?
            if *n_responses.lock().unwrap() <= BYZANTINE_THRESHOLD {
                quorum = true;
            }
        }
    }

}
    

    
    

