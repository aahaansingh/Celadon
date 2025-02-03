use serde::{Deserialize, Serialize};
use sqlx;

pub struct Folder {
    pub id: i32,
    pub name: String
}

pub struct FolderFeed {
    pub folder_id: i32,
    pub feed_id: i32
}