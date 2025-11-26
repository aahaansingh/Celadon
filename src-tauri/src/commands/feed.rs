use crate::api::feed_api;
use crate::models::{article, feed};
use sea_orm::DatabaseConnection;
use tauri::State;

#[tauri::command]
pub async fn get_feed(state: State<'_, DatabaseConnection>, id: i32) -> Result<feed::Model, String> {
    let db = state.inner();
    feed_api::get_feed(db, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_feeds(
    state: State<'_, DatabaseConnection>,
) -> Result<Vec<feed::Model>, String> {
    let db = state.inner();
    feed_api::get_all_feeds(db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_feed_by_url(
    state: State<'_, DatabaseConnection>,
    url: String,
) -> Result<Option<feed::Model>, String> {
    let db = state.inner();
    feed_api::get_feed_by_url(db, url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_feed_name(
    state: State<'_, DatabaseConnection>,
    id: i32,
    name: String,
) -> Result<(), String> {
    let db = state.inner();
    feed_api::update_feed_str(db, id, feed_api::FeedStrFields::Name, name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_feed_folder(
    state: State<'_, DatabaseConnection>,
    id: i32,
    folder_id: i32,
) -> Result<(), String> {
    let db = state.inner();
    feed_api::update_feed_folder(db, id, folder_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_feed(state: State<'_, DatabaseConnection>, id: i32) -> Result<(), String> {
    let db = state.inner();
    feed_api::delete_feed(db, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_articles(
    state: State<'_, DatabaseConnection>,
    id: i32,
    num: Option<u64>,
) -> Result<Vec<article::Model>, String> {
    let db = state.inner();
    feed_api::get_articles(db, id, num)
        .await
        .map_err(|e| e.to_string())
}
