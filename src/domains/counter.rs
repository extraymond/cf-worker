use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Counter {
    pub name: String,
    pub count: i32,
}
