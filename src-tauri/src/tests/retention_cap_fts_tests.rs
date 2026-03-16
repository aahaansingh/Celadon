//! Tests for retention (delete old untagged), article cap, and FTS5 search.

use crate::api::{article_api, feed_api, superfeed_api, tag_api};
use crate::models::article::ReadFilter;
use crate::models::{article, create_tables, feed};
use chrono::{TimeDelta, Utc};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::utils::TestDB;

static DB_COUNTER: AtomicUsize = AtomicUsize::new(0);

async fn test_db() -> TestDB {
    let n = DB_COUNTER.fetch_add(1, Ordering::SeqCst);
    TestDB::new(&format!("retention_cap_fts_{}", n)).await
}

#[async_std::test]
async fn test_retention_deletes_old_untagged_keeps_tagged() -> Result<(), Box<dyn std::error::Error>>
{
    let test_db = test_db().await;
    create_tables::create_tables(&test_db.db).await?;
    let db = &test_db.db;

    superfeed_api::create_superfeed(db, 1, "All".to_owned()).await?;
    feed_api::create_feed(
        db,
        1,
        "https://example.com/feed".to_owned(),
        "Test Feed".to_owned(),
        "".to_owned(),
        Utc::now(),
        Utc::now(),
        0,
        feed::FeedType::Article,
    )
    .await?;
    feed_api::add_feed_to_superfeed(db, 1, 1).await?;

    let old = Utc::now() - TimeDelta::days(400);
    let expiry = old + TimeDelta::days(3);

    // Article 1: old, untagged -> should be deleted
    article_api::create_article(
        db,
        1,
        "https://example.com/1".to_owned(),
        "Old Untagged".to_owned(),
        old,
        expiry,
        true,
        "desc".to_owned(),
        1,
    )
    .await?;
    // Article 2: old, tagged -> should be kept
    article_api::create_article(
        db,
        2,
        "https://example.com/2".to_owned(),
        "Old Tagged".to_owned(),
        old,
        expiry,
        true,
        "desc".to_owned(),
        1,
    )
    .await?;
    tag_api::create_tag(db, 1, "Keep".to_owned()).await?;
    tag_api::tag_article(db, 1, 2).await?;

    let deleted = article_api::delete_articles_older_than_retention(db).await?;
    assert_eq!(deleted, 1, "Should delete exactly one old untagged article");

    let remaining = article::Entity::find()
        .filter(article::Column::Deleted.eq(false))
        .all(db)
        .await?;
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].name, "Old Tagged");

    Ok(())
}

#[async_std::test]
async fn test_ensure_article_cap_removes_oldest_untagged() -> Result<(), Box<dyn std::error::Error>>
{
    let test_db = test_db().await;
    create_tables::create_tables(&test_db.db).await?;
    let db = &test_db.db;

    superfeed_api::create_superfeed(db, 1, "All".to_owned()).await?;
    feed_api::create_feed(
        db,
        1,
        "https://example.com/feed".to_owned(),
        "Test Feed".to_owned(),
        "".to_owned(),
        Utc::now(),
        Utc::now(),
        0,
        feed::FeedType::Article,
    )
    .await?;
    feed_api::add_feed_to_superfeed(db, 1, 1).await?;

    let base = Utc::now() - TimeDelta::days(10);
    let expiry = base + TimeDelta::days(3);

    for i in 1..=5 {
        let published = base + TimeDelta::days(i);
        article_api::create_article(
            db,
            i as i32,
            format!("https://example.com/{}", i),
            format!("Article {}", i),
            published,
            expiry,
            false,
            "desc".to_owned(),
            1,
        )
        .await?;
    }
    // Tag article 2 and 4 so they are kept; we'll cap at 3, so oldest untagged (1,3,5) -> delete 1 and 3 (two oldest), keep 2,4,5
    tag_api::create_tag(db, 1, "T".to_owned()).await?;
    tag_api::tag_article(db, 1, 2).await?;
    tag_api::tag_article(db, 1, 4).await?;

    let removed = article_api::ensure_article_cap(db, 3).await?;
    assert_eq!(removed, 2, "Should remove 2 to get from 5 down to 3");

    let count = article_api::article_count(db).await?;
    assert_eq!(count, 3);

    let remaining = article::Entity::find()
        .filter(article::Column::Deleted.eq(false))
        .all(db)
        .await?;
    let names: Vec<String> = remaining.iter().map(|a| a.name.clone()).collect();
    assert!(names.contains(&"Article 2".to_owned()));
    assert!(names.contains(&"Article 4".to_owned()));
    assert!(names.contains(&"Article 5".to_owned()));

    Ok(())
}

#[async_std::test]
async fn test_fts_search_articles_feeds_superfeeds_tags() -> Result<(), Box<dyn std::error::Error>>
{
    let test_db = test_db().await;
    create_tables::create_tables(&test_db.db).await?;
    let db = &test_db.db;

    superfeed_api::create_superfeed(db, 1, "All".to_owned()).await?;
    superfeed_api::create_superfeed(db, 2, "UniqueSuperfeedName".to_owned()).await?;
    feed_api::create_feed(
        db,
        1,
        "https://example.com/feed".to_owned(),
        "UniqueFeedName".to_owned(),
        "".to_owned(),
        Utc::now(),
        Utc::now(),
        0,
        feed::FeedType::Article,
    )
    .await?;
    feed_api::add_feed_to_superfeed(db, 1, 1).await?;
    tag_api::create_tag(db, 1, "UniqueTagName".to_owned()).await?;

    let now = Utc::now();
    let expiry = now + TimeDelta::days(3);
    article_api::create_article(
        db,
        1,
        "https://example.com/1".to_owned(),
        "UniqueArticleTitle".to_owned(),
        now,
        expiry,
        false,
        "UniqueArticleDescription".to_owned(),
        1,
    )
    .await?;

    let articles = article_api::search_articles(
        db,
        "UniqueArticleTitle".to_owned(),
        ReadFilter::All,
        Some(10),
        None,
    )
    .await?;
    assert!(!articles.is_empty());
    assert!(articles
        .iter()
        .any(|a| a.name.contains("UniqueArticleTitle")));

    let feeds = feed_api::search_feeds(db, "UniqueFeedName".to_owned()).await?;
    assert!(!feeds.is_empty());
    assert!(feeds.iter().any(|f| f.name == "UniqueFeedName"));

    let superfeeds = superfeed_api::search_superfeeds(db, "UniqueSuperfeedName".to_owned()).await?;
    assert!(!superfeeds.is_empty());
    assert!(superfeeds.iter().any(|s| s.name == "UniqueSuperfeedName"));

    let tags = tag_api::search_tags(db, "UniqueTagName".to_owned()).await?;
    assert!(!tags.is_empty());
    assert!(tags.iter().any(|t| t.name == "UniqueTagName"));

    Ok(())
}
