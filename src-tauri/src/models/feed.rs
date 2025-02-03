use serde::{Deserialize, Serialize};
use image::ImageReader;
use chrono::DateTime;
use sqlx;

pub struct Feed{
    pub id: i32,
    pub url: String,
    pub name: String,
    pub category: String,
    pub last_fetched: DateTime,
    pub healthy: bool
}



