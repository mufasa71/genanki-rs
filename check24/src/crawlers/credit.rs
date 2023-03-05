use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Credit {
    pub title: String,
    pub rate: String,
    pub term: String,
    pub sum: String,
}
