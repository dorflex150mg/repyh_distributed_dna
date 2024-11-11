use std::sync::Arc;
use serde::{Serialize, Deserialize};

/// Structure for DNA sequence patch data.
#[derive(Serialize, Deserialize, Clone)]
pub struct Patch { 
    pub id: Arc<str>,
    pub patch_txt: Arc<str>,
}

impl Patch {
    pub fn new(id: Arc<str>, patch_txt: Arc<str>) -> Self {
        Patch {
            id,
            patch_txt,
        }
    }
}
