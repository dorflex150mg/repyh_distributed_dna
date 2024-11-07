use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::fmt::{self, Display};

#[derive(Serialize, Deserialize, Clone)]
pub struct DnaSequence {
    pub id: String,//TODO: make private, create builder and "with_id" method
    pub dna_sequence: String,
}

impl Display for DnaSequence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{}, {}}}", self.id, self.dna_sequence) 
    }
}
        

impl DnaSequence {
    pub fn new(dna_sequence: String) -> Self { 
        let id = Uuid::new_v4().to_string();
        DnaSequence{
            id, 
            dna_sequence,
        }
    }
}