// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod api;
mod commands;
mod models;
mod syndication;
mod tests;
mod undo;

use undo::UndoStack;

use sea_orm::{Database, DatabaseConnection};
use std::fs::create_dir_all;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .manage(UndoStack::new())
        .setup(|app| {
            let app_data_dir = app.path_resolver().app_data_dir().unwrap();
            if !app_data_dir.exists() {
                create_dir_all(&app_data_dir).unwrap();
            }
            let db_path = app_data_dir.join("celadon.db");
            let db_url = format!("sqlite:{}", db_path.to_str().unwrap());

            let db_conn = tauri::async_runtime::block_on(async {
                Database::connect(&db_url)
                    .await
                    .expect("database connection failed")
            });

            tauri::async_runtime::block_on(async {
                models::create_tables::create_tables(&db_conn)
                    .await
                    .expect("table creation failed");
                api::article_api::cleanup_deleted_articles(&db_conn)
                    .await
                    .unwrap_or(());
                api::feed_api::cleanup_deleted_feeds(&db_conn)
                    .await
                    .unwrap_or(());
                api::superfeed_api::cleanup_deleted_superfeeds(&db_conn)
                    .await
                    .unwrap_or(());
                api::tag_api::cleanup_deleted_tags(&db_conn)
                    .await
                    .unwrap_or(());
            });

            app.manage(db_conn);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::feed::get_feed,
            commands::feed::get_all_feeds,
            commands::feed::get_feed_by_url,
            commands::feed::update_feed_name,
            commands::feed::get_articles,
            commands::feed::add_feed_to_superfeed,
            commands::feed::remove_feed_from_superfeed,
            commands::feed::delete_feed,
            // superfeed commands
            commands::superfeed::get_superfeed,
            commands::superfeed::get_all_superfeeds,
            commands::superfeed::create_superfeed,
            commands::superfeed::rename_superfeed,
            commands::superfeed::delete_superfeed,
            commands::superfeed::get_superfeed_feeds,
            // article commands
            commands::article::get_article,
            commands::article::get_article_by_url,
            commands::article::read_article,
            commands::article::unread_article,
            commands::article::delete_article,
            commands::article::read_all_articles_in_feed,
            commands::article::get_article_tags,
            // tag commands
            commands::tag::get_tag,
            commands::tag::get_all_tags,
            commands::tag::create_tag,
            commands::tag::rename_tag,
            commands::tag::delete_tag,
            commands::tag::tag_article,
            commands::tag::untag_article,
            commands::tag::get_tagged_articles,
            // syndication commands
            commands::syndication::add_feed,
            // undo commands
            commands::undo::undo,
            commands::undo::clear_undo,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
