use crate::api::article_api;
use crate::models::{article, tag};
use sea_orm::DatabaseConnection;
use tauri::State;

#[tauri::command]
pub async fn get_article(
    state: State<'_, DatabaseConnection>,
    id: i32,
) -> Result<article::Model, String> {
    let db = state.inner();
    article_api::get_article(db, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_article_by_url(
    state: State<'_, DatabaseConnection>,
    url: String,
) -> Result<Option<article::Model>, String> {
    let db = state.inner();
    article_api::get_article_by_url(db, url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_article(state: State<'_, DatabaseConnection>, id: i32) -> Result<(), String> {
    let db = state.inner();
    article_api::read_article(db, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_all_articles_in_feed(
    state: State<'_, DatabaseConnection>,
    feed_id: i32,
) -> Result<(), String> {
    let db = state.inner();
    article_api::read_all(db, feed_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_article_tags(
    state: State<'_, DatabaseConnection>,
    id: i32,
) -> Result<Vec<tag::Model>, String> {
    let db = state.inner();
    article_api::get_tags(db, id)
        .await
        .map_err(|e| e.to_string())
}
