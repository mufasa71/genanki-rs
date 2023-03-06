use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum CreditType {
    Auto,
    Mortgage,
    Micro,
    Education,
    Consumer,
    Overdraft,
    CreditCard,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credit {
    pub credit_type: CreditType,
    pub title: String,
    pub rate: String,
    pub term: String,
    pub sum: String,
}
