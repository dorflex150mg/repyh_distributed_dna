use std::sync::Arc;
use serde::{Serialize, Deserialize};
use std::fmt::{self, Display};

/// Structure representing a DNA sequence.
#[derive(Serialize, Deserialize, Clone)]
pub struct DnaSequence {
    pub id: Arc<str>, // Unique identifier for the DNA sequence.
    pub dna_sequence: Arc<str>, // The DNA sequence data.
}

impl Display for DnaSequence {
    /// Formats the DNA sequence for display.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{}, {}}}", self.id, self.dna_sequence)
    }
}

impl DnaSequence {
    /// Creates a new DNA sequence with the given ID and sequence data.
    pub fn new(id: Arc<str>, dna_sequence: Arc<str>) -> Self {
        DnaSequence {
            id,
            dna_sequence,
        }
    }
}

