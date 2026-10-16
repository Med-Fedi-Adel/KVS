use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Request {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    Ok,
    Value(Option<String>),
    Error(String),
}
