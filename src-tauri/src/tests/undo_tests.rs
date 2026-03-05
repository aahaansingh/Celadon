use super::utils::{self, TestDB};
use crate::api::*;
use crate::models::{create_tables, feed};
use chrono::Utc;
use sea_orm::{DbConn, DbErr};

#[async_std::test]
async fn test_undo_functionality() -> Result<(), DbErr> {
    let test_db = TestDB::new("sql_test_undo").await;
    let db = &test_db.db;
    create_tables::create_tables(db).await?;

    let add = Utc::now();
    let fetch = Utc::now();

    // Setup initial data
    superfeed_api::create_superfeed(db, 1, "main".to_owned()).await?;
    feed_api::create_feed(
        db,
        1,
        "url".to_owned(),
        "name".to_owned(),
        "cat".to_owned(),
        add,
        fetch,
        true,
        feed::FeedType::News,
    )
    .await?;
    article_api::create_article(
        db,
        1,
        "url".to_owned(),
        "name".to_owned(),
        add,
        fetch,
        false,
        "desc".to_owned(),
        1,
    )
    .await?;
    tag_api::create_tag(db, 1, "mytag".to_owned()).await?;

    // --- Test Article Read/Unread ---
    article_api::read_article(db, 1).await?;
    let article = article_api::get_article(db, 1).await?;
    assert_eq!(article.read, true);

    article_api::unread_article(db, 1).await?;
    let article = article_api::get_article(db, 1).await?;
    assert_eq!(article.read, false);

    // --- Test Soft Deletion / Undeletion ---

    // Article
    article_api::delete_article(db, 1).await?;
    assert!(article_api::get_article(db, 1).await.is_err());
    article_api::undelete_article(db, 1).await?;
    assert!(article_api::get_article(db, 1).await.is_ok());

    // Tag
    tag_api::delete_tag(db, 1).await?;
    assert!(tag_api::get_tag(db, 1).await.is_err());
    tag_api::undelete_tag(db, 1).await?;
    assert!(tag_api::get_tag(db, 1).await.is_ok());

    // Feed
    feed_api::delete_feed(db, 1).await?;
    assert!(feed_api::get_feed(db, 1).await.is_err());
    feed_api::undelete_feed(db, 1).await?;
    assert!(feed_api::get_feed(db, 1).await.is_ok());

    // Superfeed (Testing secondary superfeed as 1 is protected)
    superfeed_api::create_superfeed(db, 2, "secondary".to_owned()).await?;
    superfeed_api::delete_superfeed(db, 2).await?;
    assert!(superfeed_api::get_superfeed(db, 2).await.is_err());
    superfeed_api::undelete_superfeed(db, 2).await?;
    assert!(superfeed_api::get_superfeed(db, 2).await.is_ok());

    Ok(())
}
