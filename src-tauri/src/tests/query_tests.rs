use super::utils::TestDB;
use crate::api::{article_api, feed_api, superfeed_api, tag_api};
use crate::models::article::ReadFilter;
use crate::models::{article, create_tables, feed};
use crate::syndication::syndicator::{self, SyndicationFeed};
use sea_orm::{DbConn, EntityTrait};
use std::fs;
use std::path::PathBuf;

#[async_std::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_db = TestDB::new("query_test").await;
    create_tables::create_tables(&test_db.db).await?;

    // Seed data from test_items/feeds
    seed_from_test_items(&test_db.db).await?;

    test_feed_queries(&test_db.db).await?;
    test_superfeed_queries(&test_db.db).await?;
    test_tag_queries(&test_db.db).await?;
    test_pagination(&test_db.db).await?;

    Ok(())
}

async fn seed_from_test_items(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a "Test Superfeed"
    let superfeed_id = 1;
    superfeed_api::create_superfeed(db, superfeed_id, "Test Superfeed".to_owned()).await?;

    // 2. Identify the feeds directory
    let mut feeds_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    feeds_path.push("src/tests/test_items/feeds");

    // 3. Iterate and seed
    let entries = fs::read_dir(feeds_path)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && !path.file_name().unwrap().to_str().unwrap().starts_with('.') {
            let content = fs::read(&path)?;

            // Try Atom first, then RSS
            let feed_obj = match atom_syndication::Feed::read_from(&content[..]) {
                Ok(obj) => SyndicationFeed::Atom(obj),
                Err(_) => match rss::Channel::read_from(&content[..]) {
                    Ok(obj) => SyndicationFeed::Rss(obj),
                    Err(e) => {
                        println!("Skipping invalid feed file {:?}: {}", path, e);
                        continue;
                    }
                },
            };

            let file_url = format!("file://{}", path.to_str().unwrap());
            syndicator::new_feed(
                db,
                feed_obj,
                file_url,
                superfeed_id,
                feed::FeedType::Article,
            )
            .await?;
        }
    }

    // 4. Create some read articles and tags for testing filters
    // Mark the first article of the first feed as read
    let all_articles = article::Entity::find().all(db).await?;
    if !all_articles.is_empty() {
        article_api::read_article(db, all_articles[0].id).await?;

        // Create a tag and tag the second article
        tag_api::create_tag(db, 1, "Testing".to_owned()).await?;
        if all_articles.len() > 1 {
            tag_api::tag_article(db, 1, all_articles[1].id).await?;
        }
    }

    Ok(())
}

async fn test_feed_queries(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    let all_feeds = feed_api::get_all_feeds(db).await?;
    assert!(!all_feeds.is_empty(), "Feeds should have been seeded");

    let first_feed_id = all_feeds[0].id;

    // Testing \f:[ID] \a
    let articles = feed_api::get_articles(db, first_feed_id, ReadFilter::All, None, None).await?;
    assert!(!articles.is_empty(), "Feed should have articles");

    // Testing \f:[ID] \r
    let read_articles =
        feed_api::get_articles(db, first_feed_id, ReadFilter::Read, None, None).await?;
    assert_eq!(
        read_articles.len(),
        1,
        "Should have exactly 1 read article in the first feed"
    );

    Ok(())
}

async fn test_superfeed_queries(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    // Testing \s:1 \a (All in superfeed 1)
    let articles = superfeed_api::get_articles(db, 1, ReadFilter::All, None, None).await?;
    let db_articles = article::Entity::find().all(db).await?;
    assert_eq!(
        articles.len(),
        db_articles.len(),
        "Superfeed 1 should contain all seeded articles"
    );

    Ok(())
}

async fn test_tag_queries(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    // Testing \t:1 \a (All in tag 1)
    let articles = tag_api::get_articles(db, 1, ReadFilter::All, None, None).await?;
    assert_eq!(articles.len(), 1, "Tag 1 should have 1 article");

    Ok(())
}

async fn test_pagination(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    let page_size: u64 = 2; // Using smaller page size for the seeded articles
                            // Page 1
    let articles_p1 =
        superfeed_api::get_articles(db, 1, ReadFilter::All, Some(page_size), Some(0)).await?;
    assert_eq!(articles_p1.len() as u64, page_size);

    // Page 2
    let articles_p2 =
        superfeed_api::get_articles(db, 1, ReadFilter::All, Some(page_size), Some(page_size))
            .await?;
    // We expect some articles in page 2 if we have enough
    assert!(!articles_p2.is_empty());

    // Verify they are different
    assert_ne!(articles_p1[0].id, articles_p2[0].id);

    Ok(())
}
