pub mod dna_client {

    use ring::rand::SystemRandom;
    use ring::signature::{KeyPair, Ed25519KeyPair};

    pub struct DnaClient {
        pub key_pair: Ed25519KeyPair,
        pub dna_sequence: String,
    }


    fn generate_key_pair() -> (Ed25519KeyPair, SystemRandom) {
        let rng = SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
        .unwrap();  
        (key_pair, rng)
    }

    impl DnaClient {
        pub fn new(dna_sequence: impl Into<String>) -> Self{
            let (key_pair, _) = generate_key_pair();
            DnaClient {
                dna_sequence: dna_sequence.into(),
                key_pair,
            }
        }

        pub fn get_pub_key(&self) -> Vec<u8> {
            self.key_pair.public_key().as_ref().to_vec().clone() 
        }

        pub fn sign(&self) -> Vec<u8> {
            let dna_bytes = self.dna_sequence.as_bytes();
            let signature = self
                .key_pair
                .sign(dna_bytes)
                .as_ref()
                .to_vec();
            println!("signature: {:?}", &signature);
            signature
        }

        pub fn set_dna_sequence(&mut self, dna_sequence: impl Into<String>) {
            self.dna_sequence = dna_sequence.into();
        }
    }
}

    
