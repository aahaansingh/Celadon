use super::utils::{self, TestDB};
use crate::api::*;
use crate::models::*;
use chrono::Utc;
use sea_orm::{entity::*, error::*, query::*, sea_query, tests_cfg::*, Database, DbConn};

#[async_std::test]
async fn main() -> Result<(), DbErr> {
    let test_db = TestDB::new("sql_test").await;
    create_tables::create_tables(&test_db.db).await?;
    folder_api_test(&test_db.db).await?;
    article_api_test(&test_db.db).await?;
    Ok(())
}

async fn folder_api_test(db: &DbConn) -> Result<(), DbErr> {
    let add = Utc::now();
    let fetch = Utc::now();

    let new_folder_id = folder_api::folder_max_id(db).await? + 1;
    assert_eq!(new_folder_id, 1);
    folder_api::create_folder(db, new_folder_id, "main".to_owned()).await;

    let retrieved_folder = folder_api::get_folder(db, new_folder_id).await?;
    assert_eq!(retrieved_folder.name, "main".to_owned());
    
    let new_feed_id = feed_api::feed_max_id(db).await? + 1;
    assert_eq!(new_feed_id, 1);
    feed_api::create_feed(
        db,
        new_feed_id,
        "http://www.ocearch.org/feed".to_owned(),
        "OCearch".to_owned(),
        "News".to_owned(),
        add,
        fetch,
        true,
        retrieved_folder.id,
    )
    .await;

    let add_next = Utc::now();
    let fetch_next = Utc::now();

    let next_feed_id = feed_api::feed_max_id(db).await? + 1;
    assert_eq!(next_feed_id, 2);
    feed_api::create_feed(
        db,
        next_feed_id,
        "https://feeds.kottke.org/main".to_owned(),
        "Kottke".to_owned(),
        "Lifestyle".to_owned(),
        add_next,
        fetch_next,
        true,
        retrieved_folder.id,
    )
    .await;

    let add_last = Utc::now();
    let fetch_last = Utc::now();
    feed_api::create_feed(
        db,
        feed_api::feed_max_id(db).await? + 1,
        "https://thequietus.com/feed/".to_owned(),
        "The Quietus".to_owned(),
        "Music".to_owned(),
        add_last,
        fetch_last,
        true,
        retrieved_folder.id,
    )
    .await;

    let all_retrieved_feeds = folder_api::get_feeds(db, retrieved_folder.id, None).await?;
    assert_eq!(all_retrieved_feeds[0].id, 3);
    assert_eq!(all_retrieved_feeds[1].id, 2);
    assert_eq!(all_retrieved_feeds[2].id, 1);
    let some_retrieved_feeds = folder_api::get_feeds(db, retrieved_folder.id, Some(2)).await?;
    assert_eq!(some_retrieved_feeds.len(), 2);
    assert_eq!(some_retrieved_feeds[0].id, 3);
    assert_eq!(some_retrieved_feeds[1].id, 2);
    Ok(())
}

async fn article_api_test(db: &DbConn) -> Result<(), DbErr> {
    let empty_related_articles = feed_api::get_articles(db, 1, None).await?;
    assert_eq!(empty_related_articles.len(), 0);
    Ok(())
}
