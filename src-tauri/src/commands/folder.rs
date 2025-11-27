use crate::api::folder_api;
use crate::models::{feed, folder};
use sea_orm::DatabaseConnection;
use tauri::State;

#[tauri::command]
pub async fn get_folder(
    state: State<'_, DatabaseConnection>,
    id: i32,
) -> Result<folder::Model, String> {
    let db = state.inner();
    folder_api::get_folder(db, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_folders(
    state: State<'_, DatabaseConnection>,
) -> Result<Vec<folder::Model>, String> {
    let db = state.inner();
    folder_api::get_all_feeds(db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_folder(
    state: State<'_, DatabaseConnection>,
    name: String,
) -> Result<(), String> {
    let db = state.inner();
    let id = folder_api::folder_max_id(db)
        .await
        .map_err(|e| e.to_string())?
        + 1;
    folder_api::create_folder(db, id, name)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn rename_folder(
    state: State<'_, DatabaseConnection>,
    id: i32,
    name: String,
) -> Result<(), String> {
    let db = state.inner();
    folder_api::rename_folder(db, id, name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_folder(state: State<'_, DatabaseConnection>, id: i32) -> Result<(), String> {
    let db = state.inner();
    folder_api::delete_folder(db, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_folder_feeds(
    state: State<'_, DatabaseConnection>,
    id: i32,
    num: Option<u64>,
) -> Result<Vec<feed::Model>, String> {
    let db = state.inner();
    folder_api::get_feeds(db, id, num)
        .await
        .map_err(|e| e.to_string())
}
