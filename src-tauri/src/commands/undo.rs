use crate::api::{article_api, feed_api, superfeed_api, tag_api};
use crate::undo::{Action, UndoStack};
use sea_orm::DatabaseConnection;
use tauri::State;

#[tauri::command]
pub async fn undo(
    state: State<'_, UndoStack>,
    db: State<'_, DatabaseConnection>,
) -> Result<Option<Action>, String> {
    let action_opt = state.pop();
    let conn = db.inner();
    if let Some(action) = action_opt.clone() {
        match action {
            Action::MarkRead(id) => {
                article_api::unread_article(conn, id)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            Action::MarkUnread(id) => {
                article_api::read_article(conn, id)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            Action::DeleteArticle(id) => {
                article_api::undelete_article(conn, id)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            Action::DeleteFeed(id) => {
                feed_api::undelete_feed(conn, id)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            Action::DeleteSuperfeed(id) => {
                superfeed_api::undelete_superfeed(conn, id)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            Action::DeleteTag(id) => {
                tag_api::undelete_tag(conn, id)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(action_opt)
}

#[tauri::command]
pub async fn clear_undo(state: State<'_, UndoStack>) -> Result<(), String> {
    state.clear();
    Ok(())
}

pub async fn handle_dropped_action(db: &DatabaseConnection, action: Action) {
    match action {
        Action::DeleteArticle(id) => {
            let _ = article_api::hard_delete_article(db, id).await;
        }
        Action::DeleteFeed(id) => {
            let _ = feed_api::hard_delete_feed(db, id).await;
        }
        Action::DeleteSuperfeed(id) => {
            let _ = superfeed_api::hard_delete_superfeed(db, id).await;
        }
        Action::DeleteTag(id) => {
            let _ = tag_api::hard_delete_tag(db, id).await;
        }
        _ => {}
    }
}
