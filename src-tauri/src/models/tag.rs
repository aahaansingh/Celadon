use serde::{Deserialize, Serialize};
use sqlx;

pub struct Tag {
    pub id: i32,
    pub name: String
}

pub struct TagArticle {
    pub tag_id: i32,
    pub article_id: i32
}