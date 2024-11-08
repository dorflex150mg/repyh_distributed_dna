use serde::{Serialize, Deserialize};
use std::fmt::{self, Display};

/// Structure representing a DNA sequence.
#[derive(Serialize, Deserialize, Clone)]
pub struct DnaSequence {
    pub id: String, // Unique identifier for the DNA sequence.
    pub dna_sequence: String, // The DNA sequence data.
}

impl Display for DnaSequence {
    /// Formats the DNA sequence for display.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{}, {}}}", self.id, self.dna_sequence)
    }
}

impl DnaSequence {
    /// Creates a new DNA sequence with the given ID and sequence data.
    pub fn new(id: String, dna_sequence: String) -> Self {
        DnaSequence {
            id,
            dna_sequence,
        }
    }
}

