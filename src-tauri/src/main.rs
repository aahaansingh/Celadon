// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod api;
mod commands;
mod models;
mod syndication;
mod tests;

use sea_orm::{Database, DatabaseConnection};
use std::fs::create_dir_all;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
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
            });

            app.manage(db_conn);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::feed::get_feed,
            commands::feed::get_all_feeds,
            commands::feed::get_feed_by_url,
            commands::feed::update_feed_name,
            commands::feed::update_feed_folder,
            commands::feed::delete_feed,
            commands::feed::get_articles,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
