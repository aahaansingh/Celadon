use crate::api::{feed_api, folder_api};
use crate::models::create_tables;
use crate::syndication::syndicator;
use sea_orm::DbConn;

use super::utils::TestDB;

#[async_std::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_db = TestDB::new("syndication_test").await;
    create_tables::create_tables(&test_db.db).await?;
    test_add_and_update_atom_feed(&test_db.db).await?;
    test_add_rss_feed(&test_db.db).await?;
    Ok(())
}

async fn test_add_and_update_atom_feed(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    let main_folder_id = folder_api::folder_max_id(db).await? + 1;
    folder_api::create_folder(db, main_folder_id, "main".to_owned()).await?;

    let feed_url = "https://feeds.kottke.org/main".to_owned();
    syndicator::url_to_feed(db, feed_url.clone(), main_folder_id).await?;

    assert_eq!(feed_api::feed_max_id(db).await?, 1);
    let inserted_feed = feed_api::get_feed(db, 1).await?;
    assert_eq!(inserted_feed.name, "kottke.org");

    let retrieved_feed = feed_api::get_feed_by_url(db, feed_url.clone())
        .await?
        .unwrap();
    assert_eq!(retrieved_feed.id, 1);

    let retrieved_articles = feed_api::get_articles(db, 1, None).await?;
    let initial_article_count = retrieved_articles.len();
    assert!(initial_article_count > 0);

    let initial_fetch_time = retrieved_feed.last_fetched;
    // Ensure some time passes before next fetch
    std::thread::sleep(std::time::Duration::from_secs(1));

    syndicator::url_to_feed(db, feed_url.clone(), main_folder_id).await?;

    let updated_feed = feed_api::get_feed(db, 1).await?;
    assert!(updated_feed.last_fetched > initial_fetch_time);

    let newly_retrieved_articles = feed_api::get_articles(db, 1, None).await?;
    assert_eq!(initial_article_count, newly_retrieved_articles.len());
    assert_eq!(feed_api::feed_max_id(db).await?, 1);

    Ok(())
}

async fn test_add_rss_feed(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(folder_api::folder_max_id(db).await?, 1);
    let main_folder_id = 1;

    let feed_url = "https://aahaansingh.github.io/posts/index.xml".to_owned();
    syndicator::url_to_feed(db, feed_url.clone(), main_folder_id).await?;

    assert_eq!(feed_api::feed_max_id(db).await?, 2);
    let inserted_feed = feed_api::get_feed(db, 2).await?;
    assert_eq!(inserted_feed.name, "Posts on Aahaan Singh");

    let retrieved_feed = feed_api::get_feed_by_url(db, feed_url)
        .await?
        .unwrap();
    assert_eq!(retrieved_feed.id, 2);

    let articles = feed_api::get_articles(db, 2, None).await?;
    assert!(articles.len() > 0);

    Ok(())
}
