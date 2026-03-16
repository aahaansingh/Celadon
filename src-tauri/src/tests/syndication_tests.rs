use crate::api::{feed_api, superfeed_api};
use crate::models::article::ReadFilter;
use crate::models::{create_tables, feed};
use crate::syndication::syndicator;
use sea_orm::DbConn;

use super::utils::TestDB;

#[async_std::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_db = TestDB::new("syndication_test").await;
    create_tables::create_tables(&test_db.db).await?;
    test_add_and_update_atom_feed(&test_db.db).await?;
    test_add_rss_feed(&test_db.db).await?;
    test_substack_feed(&test_db.db).await?;
    Ok(())
}

async fn test_add_and_update_atom_feed(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    let main_superfeed_id = superfeed_api::superfeed_max_id(db).await? + 1;
    superfeed_api::create_superfeed(db, main_superfeed_id, "main".to_owned()).await?;

    let feed_url = "https://feeds.kottke.org/main".to_owned();
    syndicator::url_to_feed(
        db,
        feed_url.clone(),
        main_superfeed_id,
        feed::FeedType::Article,
    )
    .await?;

    assert_eq!(feed_api::feed_max_id(db).await?, 1);
    let inserted_feed = feed_api::get_feed(db, 1).await?;
    assert_eq!(inserted_feed.name, "kottke.org");

    let retrieved_feed = feed_api::get_feed_by_url(db, feed_url.clone())
        .await?
        .unwrap();
    assert_eq!(retrieved_feed.id, 1);

    let retrieved_articles = feed_api::get_articles(db, 1, ReadFilter::All, None, None).await?;
    let initial_article_count = retrieved_articles.len();
    assert!(initial_article_count > 0);

    let initial_fetch_time = retrieved_feed.last_fetched;
    // Ensure some time passes before next fetch
    std::thread::sleep(std::time::Duration::from_secs(1));

    syndicator::url_to_feed(
        db,
        feed_url.clone(),
        main_superfeed_id,
        feed::FeedType::Article,
    )
    .await?;

    let updated_feed = feed_api::get_feed(db, 1).await?;
    assert!(updated_feed.last_fetched > initial_fetch_time);

    let newly_retrieved_articles =
        feed_api::get_articles(db, 1, ReadFilter::All, None, None).await?;
    assert_eq!(initial_article_count, newly_retrieved_articles.len());
    assert_eq!(feed_api::feed_max_id(db).await?, 1);

    Ok(())
}

async fn test_add_rss_feed(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    let main_superfeed_id = 1;

    let feed_url = "https://aahaansingh.github.io/posts/index.xml".to_owned();
    syndicator::url_to_feed(
        db,
        feed_url.clone(),
        main_superfeed_id,
        feed::FeedType::Article,
    )
    .await?;

    assert_eq!(feed_api::feed_max_id(db).await?, 2);
    let inserted_feed = feed_api::get_feed(db, 2).await?;
    assert_eq!(inserted_feed.name, "Posts on Aahaan Singh");

    let retrieved_feed = feed_api::get_feed_by_url(db, feed_url).await?.unwrap();
    assert_eq!(retrieved_feed.id, 2);

    let articles = feed_api::get_articles(db, 2, ReadFilter::All, None, None).await?;
    assert!(articles.len() > 0);

    Ok(())
}

/// Substack feed (Atom): assert we parse it and get articles with readable content (e.g. summary fallback).
async fn test_substack_feed(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    let superfeed_id = 1;
    let feed_url = "https://www.thebignewsletter.com/feed".to_owned();

    syndicator::url_to_feed(db, feed_url.clone(), superfeed_id, feed::FeedType::Article).await?;

    let feed_id = feed_api::feed_max_id(db).await?;
    let inserted_feed = feed_api::get_feed(db, feed_id).await?;
    assert!(
        !inserted_feed.name.is_empty(),
        "Substack feed should have a name"
    );

    let retrieved_feed = feed_api::get_feed_by_url(db, feed_url).await?.unwrap();
    assert_eq!(retrieved_feed.id, feed_id);

    let articles = feed_api::get_articles(db, feed_id, ReadFilter::All, None, None).await?;
    assert!(
        articles.len() > 0,
        "Substack feed should yield at least one article"
    );

    let first = &articles[0];
    assert!(
        !first.description.trim().is_empty() && first.description != "No description provided.",
        "First article should have real content (title or summary), got description len {}",
        first.description.len()
    );

    Ok(())
}
