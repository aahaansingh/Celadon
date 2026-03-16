use crate::models::feed;
use crate::syndication::syndicator;
use sea_orm::DatabaseConnection;
use tauri::State;

#[tauri::command]
pub async fn add_feed(
    state: State<'_, DatabaseConnection>,
    url: String,
    superfeed_id: i32,
    feed_type: feed::FeedType,
) -> Result<(), String> {
    let db = state.inner();
    syndicator::url_to_feed(db, url, superfeed_id, feed_type)
        .await
        .map_err(|e| e.to_string())
}

/// Re-fetch all feeds from their URLs and insert new articles. Used by the hourly background task;
/// the UI refresh button should only re-read from the DB (loadData).
#[tauri::command]
pub async fn refresh_all_feeds(state: State<'_, DatabaseConnection>) -> Result<(), String> {
    let db = state.inner();
    syndicator::refresh_all_feeds(db)
        .await
        .map_err(|e| e.to_string())
}
