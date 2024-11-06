pub mod node {
 
    use ring::rand::{SystemRandom};
    use ring::signature::{KeyPair, EcdsaKeyPair, ECDSA_P256_SHA256_ASN1_SIGNING};
    use std::sync::{Arc, Mutex};
    use std::fmt;
    use uuid::Uuid;
    use std::error::Error;

    use crate::server::server::Server;

    pub struct Node {
        pub id: Uuid,
        pub key_pair: EcdsaKeyPair,
        rng: SystemRandom,
        pub server: Server,
    }

    fn generate_key_pair() -> (EcdsaKeyPair, SystemRandom) {
        let rng = SystemRandom::new();
        let pkcs8_bytes = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &rng).unwrap();
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, pkcs8_bytes.as_ref(), &rng)
        .unwrap();  
        (key_pair, rng)
    }


    impl Node {
        pub async fn new(ip: impl Into<String>, peers: Vec<String>) -> Result<Self, Box<dyn Error>> {
            let (key_pair, rng) = generate_key_pair();
            let buffer = String::new();
            let buffer_arc = Arc::new(Mutex::new(buffer));
            let buffer_clone = buffer_arc.clone();
            let mut server = Server::new(ip, peers, buffer_clone).await?;
            let _ = server.init().await;
            Ok(Node {
                id: Uuid::new_v4(),
                key_pair,
                rng,
                server,
            })
            
        }
    }

}
