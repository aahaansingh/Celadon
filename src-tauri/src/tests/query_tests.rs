use super::utils::TestDB;
use crate::api::{article_api, feed_api, opml_api, superfeed_api, tag_api};
use crate::models::article::ReadFilter;
use crate::models::{create_tables, feed};
use sea_orm::{DbConn, DbErr};
use std::env;

#[async_std::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_db = TestDB::new("query_test").await;
    create_tables::create_tables(&test_db.db).await?;

    // Seed data via OPML import
    // Note: In a real test we'd use a temporary file, but here we'll simulate it
    // or use one if available in the project structure.
    // For this test, we'll just manually seed similar to how OPML would.
    seed_data(&test_db.db).await?;

    test_feed_queries(&test_db.db).await?;
    test_superfeed_queries(&test_db.db).await?;
    test_tag_queries(&test_db.db).await?;
    test_pagination(&test_db.db).await?;

    Ok(())
}

async fn seed_data(db: &DbConn) -> Result<(), DbErr> {
    let now = chrono::Utc::now();
    let old = now - chrono::TimeDelta::days(5);

    // Create Feed 1 (News)
    feed_api::create_feed(
        db,
        1,
        "url1".to_owned(),
        "NYT".to_owned(),
        "News".to_owned(),
        now,
        now,
        true,
        feed::FeedType::News,
    )
    .await?;
    // Create Feed 2 (Article)
    feed_api::create_feed(
        db,
        2,
        "url2".to_owned(),
        "Kottke".to_owned(),
        "Blog".to_owned(),
        now,
        now,
        true,
        feed::FeedType::Article,
    )
    .await?;

    // Create Superfeed 2
    superfeed_api::create_superfeed(db, 2, "Daily".to_owned()).await?;
    feed_api::add_feed_to_superfeed(db, 1, 2).await?;
    feed_api::add_feed_to_superfeed(db, 2, 2).await?;

    // Articles for Feed 1 (NYT)
    // Article 1: Unread
    article_api::create_article(
        db,
        1,
        "l1".to_owned(),
        "NYT Unread 1".to_owned(),
        now,
        now + chrono::TimeDelta::days(1),
        false,
        "desc".to_owned(),
        1,
    )
    .await?;
    // Article 2: Read
    article_api::create_article(
        db,
        2,
        "l2".to_owned(),
        "NYT Read 1".to_owned(),
        old,
        old + chrono::TimeDelta::days(1),
        true,
        "desc".to_owned(),
        1,
    )
    .await?;

    // Articles for Feed 2 (Kottke)
    // Article 3: Unread
    article_api::create_article(
        db,
        3,
        "l3".to_owned(),
        "Kottke Unread 1".to_owned(),
        now,
        now + chrono::TimeDelta::days(3),
        false,
        "desc".to_owned(),
        2,
    )
    .await?;

    // Create Tag
    tag_api::create_tag(db, 1, "Research".to_owned()).await?;
    tag_api::tag_article(db, 1, 1).await?; // Tag Article 1

    Ok(())
}

async fn test_feed_queries(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    // \f:NYT \a (All in feed 1)
    let articles = feed_api::get_articles(db, 1, ReadFilter::All, None, None).await?;
    assert_eq!(articles.len(), 2);

    // \f:NYT \u (Unread in feed 1)
    let articles = feed_api::get_articles(db, 1, ReadFilter::Unread, None, None).await?;
    assert_eq!(articles.len(), 1);
    assert_eq!(articles[0].name, "NYT Unread 1");

    Ok(())
}

async fn test_superfeed_queries(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    // \s:Daily \a (All in superfeed 2)
    let articles = superfeed_api::get_articles(db, 2, ReadFilter::All, None, None).await?;
    assert_eq!(articles.len(), 3);

    // \s:Daily \u (Unread in superfeed 2)
    let articles = superfeed_api::get_articles(db, 2, ReadFilter::Unread, None, None).await?;
    assert_eq!(articles.len(), 2);

    Ok(())
}

async fn test_tag_queries(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    // \t:Research \a (All in tag 1)
    let articles = tag_api::get_articles(db, 1, ReadFilter::All, None, None).await?;
    assert_eq!(articles.len(), 1);
    assert_eq!(articles[0].name, "NYT Unread 1");

    Ok(())
}

async fn test_pagination(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    // All articles in superfeed 2, limit 2
    let articles = superfeed_api::get_articles(db, 2, ReadFilter::All, Some(2), None).await?;
    assert_eq!(articles.len(), 2);

    // All articles in superfeed 2, limit 2, offset 2
    let articles = superfeed_api::get_articles(db, 2, ReadFilter::All, Some(2), Some(2)).await?;
    assert_eq!(articles.len(), 1);

    Ok(())
}
