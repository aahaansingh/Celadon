use serde::{Deserialize, Serialize};
use image::ImageReader;
use chrono::DateTime;
use sqlx;

pub struct Article {
    pub id: i32,
    pub url: String,
    pub name: String,
    pub feed: i32, // Foreign
    pub published: DateTime,
    pub read: bool,
    pub description: String
}
