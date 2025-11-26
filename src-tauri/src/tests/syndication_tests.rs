use crate::api::*;
use crate::models::create_tables;
use crate::syndication::syndicator;
use sea_orm::DbConn;

use super::utils::TestDB;

#[async_std::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_db = TestDB::new("sql_test").await;
    create_tables::create_tables(&test_db.db).await?;
    syndicator_test_simple(&test_db.db).await?;
    syndicator_test_update(&test_db.db).await?;
    Ok(())
}

async fn syndicator_test_simple(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    let main_folder_id = folder_api::folder_max_id(db).await? + 1;
    folder_api::create_folder(db, main_folder_id, "main".to_owned()).await;
    syndicator::url_to_feed(
        db,
        "https://feeds.kottke.org/main".to_owned(),
        main_folder_id,
    )
    .await?;
    assert_eq!(feed_api::feed_max_id(db).await?, 1);
    let inserted_model = feed_api::get_feed(db, 1).await?;
    println!("{}", inserted_model.url);
    let retrieved_feed = feed_api::get_feed_by_url(db, "https://feeds.kottke.org/main".to_owned())
        .await?
        .unwrap();
    assert_eq!(retrieved_feed.id, 1);

    let retrieved_articles = feed_api::get_articles(db, 1, None).await?;
    syndicator::url_to_feed(
        db,
        "https://feeds.kottke.org/main".to_owned(),
        main_folder_id,
    )
    .await?;
    let newly_retrieved_articles = feed_api::get_articles(db, 1, None).await?;

    let max_feed_id = feed_api::feed_max_id(db).await?;
    assert_eq!(max_feed_id, 1);
    assert_eq!(retrieved_articles.len(), newly_retrieved_articles.len());
    assert!(retrieved_articles.len() > 0);
    Ok(())
}

async fn syndicator_test_update(db: &DbConn) -> Result<(), Box<dyn std::error::Error>> {
    let main_folder_id = folder_api::folder_max_id(db).await? + 1;
    folder_api::create_folder(db, main_folder_id, "main".to_owned()).await;
    syndicator::url_to_feed(
        db,
        "https://feeds.kottke.org/main".to_owned(),
        main_folder_id,
    )
    .await?;
    syndicator::url_to_feed(
        db,
        "https://aahaansingh.github.io/posts/index.xml".to_owned(),
        main_folder_id,
    )
    .await?;
    let inserted_model = feed_api::get_feed(db, 1).await?;
    let inserted_model_rss = feed_api::get_feed(db, 2).await?;
    println!("{}", inserted_model.url);
    println!("{}", inserted_model_rss.url);

    let retrieved_feed_atom =
        feed_api::get_feed_by_url(db, "https://feeds.kottke.org/main".to_owned())
            .await?
            .unwrap();
    let retrieved_feed_rss = feed_api::get_feed_by_url(
        db,
        "https://aahaansingh.github.io/posts/index.xml".to_owned(),
    )
    .await?
    .unwrap();

    let atom_article = feed_api::get_articles(db, 1, None).await?[0].clone();
    let rss_article = feed_api::get_articles(db, 2, None).await?[0].clone();

    let atom_obj = syndicator::url_to_obj(&"https://feeds.kottke.org/main".to_owned()).await?;
    let rss_obj =
        syndicator::url_to_obj(&"https://aahaansingh.github.io/posts/index.xml".to_owned()).await?;

    syndicator::update_feed(db, 1, atom_obj).await?;
    syndicator::update_feed(db, 2, rss_obj).await?;

    let atom_article_2 = feed_api::get_articles(db, 1, None).await?[0].clone();
    let rss_article_2 = feed_api::get_articles(db, 2, None).await?[0].clone();

    assert!(atom_article.url == atom_article_2.url);
    assert_eq!(atom_article.id, atom_article_2.id);

    assert!(rss_article.url == rss_article_2.url);
    assert_eq!(rss_article.id, rss_article_2.id);

    Ok(())
}
