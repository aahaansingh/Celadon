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
