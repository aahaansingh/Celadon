use crate::api::settings_api::{self, AppSettings, UpdateAppSettings};
use crate::protocols::proxy;
use sea_orm::DatabaseConnection;
use tauri::State;

#[tauri::command]
pub async fn get_app_settings(state: State<'_, DatabaseConnection>) -> Result<AppSettings, String> {
    settings_api::get_app_settings(state.inner())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_app_settings(
    state: State<'_, DatabaseConnection>,
    patch: UpdateAppSettings,
) -> Result<AppSettings, String> {
    settings_api::update_app_settings(state.inner(), patch)
        .await
        .map_err(|e| e.to_string())
}

/// Full `celadon://…/article/<token>` URL for the given document URL (matches HTML rewriter asset base).
#[tauri::command]
pub fn get_article_proxy_url(document_url: String) -> String {
    proxy::build_article_proxy_url(&document_url)
}
