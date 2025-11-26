use crate::api::tag_api;
use crate::models::{article, tag};
use sea_orm::DatabaseConnection;
use tauri::State;

#[tauri::command]
pub async fn get_tag(state: State<'_, DatabaseConnection>, id: i32) -> Result<tag::Model, String> {
    let db = state.inner();
    tag_api::get_tag(db, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_tags(
    state: State<'_, DatabaseConnection>,
) -> Result<Vec<tag::Model>, String> {
    let db = state.inner();
    tag_api::get_all_tags(db).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_tag(state: State<'_, DatabaseConnection>, name: String) -> Result<(), String> {
    let db = state.inner();
    let id = tag_api::tag_max_id(db)
        .await
        .map_err(|e| e.to_string())?
        + 1;
    tag_api::create_tag(db, id, name)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn tag_article(
    state: State<'_, DatabaseConnection>,
    tag_id: i32,
    article_id: i32,
) -> Result<(), String> {
    let db = state.inner();
    tag_api::tag_article(db, tag_id, article_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn untag_article(
    state: State<'_, DatabaseConnection>,
    tag_id: i32,
    article_id: i32,
) -> Result<(), String> {
    let db = state.inner();
    tag_api::delete_tag_article(db, tag_id, article_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_tag(
    state: State<'_, DatabaseConnection>,
    id: i32,
    name: String,
) -> Result<(), String> {
    let db = state.inner();
    tag_api::rename_tag(db, id, name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_tag(state: State<'_, DatabaseConnection>, id: i32) -> Result<(), String> {
    let db = state.inner();
    tag_api::delete_tag(db, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tagged_articles(
    state: State<'_, DatabaseConnection>,
    id: i32,
) -> Result<Vec<article::Model>, String> {
    let db = state.inner();
    tag_api::get_articles(db, id)
        .await
        .map_err(|e| e.to_string())
}
