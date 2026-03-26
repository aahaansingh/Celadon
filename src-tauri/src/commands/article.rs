use crate::api::article_api;
use crate::commands::undo::handle_dropped_action;
use crate::models::{article, tag};
use crate::undo::{Action, UndoStack};
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
pub async fn read_article(
    state: State<'_, DatabaseConnection>,
    undo: State<'_, UndoStack>,
    id: i32,
) -> Result<(), String> {
    let db = state.inner();
    article_api::read_article(db, id)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(dropped) = undo.push(Action::MarkRead(id)) {
        handle_dropped_action(db, dropped).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn unread_article(
    state: State<'_, DatabaseConnection>,
    undo: State<'_, UndoStack>,
    id: i32,
) -> Result<(), String> {
    let db = state.inner();
    article_api::unread_article(db, id)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(dropped) = undo.push(Action::MarkUnread(id)) {
        handle_dropped_action(db, dropped).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_article(
    state: State<'_, DatabaseConnection>,
    undo: State<'_, UndoStack>,
    id: i32,
) -> Result<(), String> {
    let db = state.inner();
    article_api::delete_article(db, id)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(dropped) = undo.push(Action::DeleteArticle(id)) {
        handle_dropped_action(db, dropped).await;
    }
    Ok(())
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

use crate::models::article::ReadFilter;

#[tauri::command]
pub async fn get_all_articles(
    state: State<'_, DatabaseConnection>,
    filter: ReadFilter,
    num: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<article::Model>, String> {
    let db = state.inner();
    article_api::get_all_articles_sorted_relative(db, filter, num, offset)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_articles(
    state: State<'_, DatabaseConnection>,
    query: String,
    filter: ReadFilter,
    num: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<article::Model>, String> {
    let db = state.inner();
    article_api::search_articles(db, query, filter, num, offset)
        .await
        .map_err(|e| e.to_string())
}

/// Mark unread articles past `expiry_at` as read (same as startup maintenance). Exposed for refresh so Unread lists update without restarting the app.
#[tauri::command]
pub async fn clean_expired_articles(state: State<'_, DatabaseConnection>) -> Result<(), String> {
    let db = state.inner();
    article_api::clean_expired_articles(db)
        .await
        .map_err(|e| e.to_string())
}
