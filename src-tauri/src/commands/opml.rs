use crate::api::opml_api;
use sea_orm::DatabaseConnection;
use tauri::State;

#[tauri::command]
pub async fn import_opml(state: State<'_, DatabaseConnection>, path: String) -> Result<(), String> {
    let db = state.inner();
    opml_api::import_opml_internal(db, path).await
}

#[tauri::command]
pub async fn export_opml(state: State<'_, DatabaseConnection>, path: String) -> Result<(), String> {
    let db = state.inner();
    opml_api::export_opml_internal(db, path).await
}
