use crate::syndication::syndicator;
use sea_orm::DatabaseConnection;
use tauri::State;

#[tauri::command]
pub async fn add_feed(
    state: State<'_, DatabaseConnection>,
    url: String,
    folder_id: i32,
) -> Result<(), String> {
    let db = state.inner();
    syndicator::url_to_feed(db, url, folder_id)
        .await
        .map_err(|e| e.to_string())
}
