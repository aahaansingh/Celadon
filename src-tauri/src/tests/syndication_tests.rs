use super::utils::{self, TestDB};
use crate::api::*;
use crate::syndication::syndicator;
use crate::models::*;
use chrono::{DateTime, Utc};
use reqwest;
use sea_orm::{entity::*, error::*, query::*, sea_query, tests_cfg::*, Database, DbConn};

#[async_std::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_db = TestDB::new("sql_test").await;
    create_tables::create_tables(&test_db.db).await?;
    syndicator_test_simple(&test_db.db).await?;
    Ok(())
}

async fn syndicator_test_simple(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    let main_folder_id = folder_api::folder_max_id(db).await? + 1;
    folder_api::create_folder(db, main_folder_id, "main".to_owned()).await;
    syndicator::url_to_feed(db, "https://yasminnair.com/feed/".to_owned(), main_folder_id).await?;
    assert_eq!(feed_api::feed_max_id(db).await?, 1);
    let inserted_model = feed_api::get_feed(db, 1).await?;
    println!("{}", inserted_model.url);
    let retrieved_feed = feed_api::get_feed_by_url(db, "https://yasminnair.com/feed/".to_owned()).await?.unwrap();
    assert_eq!(retrieved_feed.id, 1);
    Ok(())
}
