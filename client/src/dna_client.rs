pub mod dna_client {

    use ring::rand::SystemRandom;
    use ring::signature::{KeyPair, EcdsaKeyPair, ECDSA_P256_SHA256_ASN1_SIGNING};

    pub struct DnaClient {
        pub key_pair: EcdsaKeyPair,
        pub dna_sequence: String,
        rng: SystemRandom,
    }


    fn generate_key_pair() -> (EcdsaKeyPair, SystemRandom) {
        let rng = SystemRandom::new();
        let pkcs8_bytes = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &rng).unwrap();
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, pkcs8_bytes.as_ref(), &rng)
        .unwrap();  
        (key_pair, rng)
    }

    impl DnaClient {
        pub fn new(dna_sequence: impl Into<String>) -> Self{
            let (key_pair, rng) = generate_key_pair();
            DnaClient {
                dna_sequence: dna_sequence.into(),
                key_pair,
                rng,
            }
        }

        pub fn get_pub_key(&self) -> Vec<u8> {
            self.key_pair.public_key().as_ref().to_vec().clone() 
        }

        pub fn sign(&self) -> Vec<u8> {
            let dna_bytes = self.dna_sequence.as_bytes();
            self
                .key_pair
                .sign(&self.rng, dna_bytes)
                .unwrap()
                .as_ref()
                .to_vec()
        }

        pub fn set_dna_sequence(&mut self, dna_sequence: impl Into<String>) {
            self.dna_sequence = dna_sequence.into();
        }
    }
}

    
