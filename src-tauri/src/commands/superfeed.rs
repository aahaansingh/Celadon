use crate::api::superfeed_api;
use crate::commands::undo::handle_dropped_action;
use crate::models::article::ReadFilter;
use crate::models::{article, feed, superfeed};
use crate::undo::{Action, UndoStack};
use sea_orm::DatabaseConnection;
use tauri::State;

#[tauri::command]
pub async fn get_superfeed(
    state: State<'_, DatabaseConnection>,
    id: i32,
) -> Result<superfeed::Model, String> {
    let db = state.inner();
    superfeed_api::get_superfeed(db, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_superfeeds(
    state: State<'_, DatabaseConnection>,
) -> Result<Vec<superfeed::Model>, String> {
    let db = state.inner();
    superfeed_api::get_all_superfeeds(db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_superfeed(
    state: State<'_, DatabaseConnection>,
    name: String,
) -> Result<(), String> {
    let db = state.inner();
    let id = superfeed_api::superfeed_max_id(db)
        .await
        .map_err(|e| e.to_string())?
        + 1;
    superfeed_api::create_superfeed(db, id, name)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn rename_superfeed(
    state: State<'_, DatabaseConnection>,
    id: i32,
    name: String,
) -> Result<(), String> {
    let db = state.inner();
    superfeed_api::rename_superfeed(db, id, name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_superfeed(
    state: State<'_, DatabaseConnection>,
    undo: State<'_, UndoStack>,
    id: i32,
) -> Result<(), String> {
    let db = state.inner();
    superfeed_api::delete_superfeed(db, id)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(dropped) = undo.push(Action::DeleteSuperfeed(id)) {
        handle_dropped_action(db, dropped).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_superfeed_feeds(
    state: State<'_, DatabaseConnection>,
    id: i32,
    num: Option<u64>,
) -> Result<Vec<feed::Model>, String> {
    let db = state.inner();
    superfeed_api::get_feeds(db, id, num)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_superfeed_articles(
    state: State<'_, DatabaseConnection>,
    id: i32,
    filter: ReadFilter,
    num: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<article::Model>, String> {
    let db = state.inner();
    superfeed_api::get_articles(db, id, filter, num, offset)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_superfeeds(
    state: State<'_, DatabaseConnection>,
    query: String,
) -> Result<Vec<superfeed::Model>, String> {
    let db = state.inner();
    superfeed_api::search_superfeeds(db, query)
        .await
        .map_err(|e| e.to_string())
}
