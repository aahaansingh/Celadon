use serde::{Deserialize, Serialize};

pub struct Feed{
    pub id: i32,
    pub url: String,
    pub name: String,
    pub last_fetched: String
}