use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    pub sort: String,
}

impl Default for Sort {
    fn default() -> Self {
        Self {
            sort: String::from("updated"),
        }
    }
}
