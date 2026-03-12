use super::utils::TestDB;
use crate::api::{article_api, feed_api, superfeed_api, tag_api};
use crate::models::article::ReadFilter;
use crate::models::{create_tables, feed};
use chrono::Utc;
use feed_api::FeedStrFields;
use sea_orm::{DbConn, DbErr};

#[async_std::test]
async fn main() -> Result<(), DbErr> {
    let test_db = TestDB::new("sql_test").await;
    create_tables::create_tables(&test_db.db).await?;
    superfeed_api_test(&test_db.db).await?;
    article_api_test(&test_db.db).await?;
    tag_api_test(&test_db.db).await?;

    // Feed backslash text directly here
    let fetch = Utc::now();
    let add = Utc::now();
    let feed_err = feed_api::create_feed(
        &test_db.db,
        999,
        "url".to_owned(),
        "bad\\feed".to_owned(),
        "cat".to_owned(),
        add,
        fetch,
        0,
        feed::FeedType::News,
    )
    .await;
    assert!(feed_err.is_err());

    Ok(())
}

async fn superfeed_api_test(db: &DbConn) -> Result<(), DbErr> {
    let add = Utc::now();
    let fetch = Utc::now();

    let new_superfeed_id = superfeed_api::superfeed_max_id(db).await? + 1;
    assert_eq!(new_superfeed_id, 1);
    superfeed_api::create_superfeed(db, new_superfeed_id, "main".to_owned()).await?;

    let retrieved_superfeed = superfeed_api::get_superfeed(db, new_superfeed_id).await?;
    assert_eq!(retrieved_superfeed.name, "main".to_owned());

    // Backslash restriction test
    let backslash_err =
        superfeed_api::create_superfeed(db, new_superfeed_id + 1, "bad\\name".to_owned()).await;
    assert!(backslash_err.is_err());
    let rename_err =
        superfeed_api::rename_superfeed(db, new_superfeed_id, "main\\super".to_owned()).await;
    assert!(rename_err.is_err());

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
        0,
        feed::FeedType::News,
    )
    .await?;
    feed_api::add_feed_to_superfeed(db, new_feed_id, retrieved_superfeed.id).await?;

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
        0,
        feed::FeedType::Article,
    )
    .await?;
    feed_api::add_feed_to_superfeed(db, next_feed_id, retrieved_superfeed.id).await?;

    let add_last = Utc::now();
    let fetch_last = Utc::now();
    let third_feed_id = feed_api::feed_max_id(db).await? + 1;
    feed_api::create_feed(
        db,
        third_feed_id,
        "https://thequietus.com/feed/".to_owned(),
        "The Quietes".to_owned(),
        "Music".to_owned(),
        add_last,
        fetch_last,
        0,
        feed::FeedType::Essay,
    )
    .await?;
    feed_api::add_feed_to_superfeed(db, third_feed_id, retrieved_superfeed.id).await?;

    feed_api::update_feed_str(db, 3, FeedStrFields::Name, "The Quietus".to_owned()).await?;

    let all_retrieved_feeds = superfeed_api::get_feeds(db, retrieved_superfeed.id, None).await?;
    assert_eq!(all_retrieved_feeds[0].name, "The Quietus".to_owned());
    assert_eq!(all_retrieved_feeds[1].id, 2);
    assert_eq!(all_retrieved_feeds[2].id, 1);
    let some_retrieved_feeds =
        superfeed_api::get_feeds(db, retrieved_superfeed.id, Some(2)).await?;
    assert_eq!(some_retrieved_feeds.len(), 2);
    assert_eq!(some_retrieved_feeds[0].id, 3);
    assert_eq!(some_retrieved_feeds[1].id, 2);

    let second_superfeed_id = superfeed_api::superfeed_max_id(db).await? + 1;
    superfeed_api::create_superfeed(db, second_superfeed_id, "newsuperfeed".to_owned()).await?;
    // Many-to-Many update: add feed 3 to second superfeed
    feed_api::add_feed_to_superfeed(db, 3, second_superfeed_id).await?;

    let second_feeds = superfeed_api::get_feeds(db, second_superfeed_id, None).await?;
    assert_eq!(second_superfeed_id, 2);
    assert_eq!(second_feeds.len(), 1);

    let all_superfeeds = superfeed_api::get_all_superfeeds(db).await?;
    assert_eq!(all_superfeeds.len(), 2);

    superfeed_api::rename_superfeed(db, second_superfeed_id, "renamedsuperfeed".to_owned()).await?;
    let renamed_superfeed = superfeed_api::get_superfeed(db, second_superfeed_id).await?;
    assert_eq!(renamed_superfeed.name, "renamedsuperfeed");

    // Deleting superfeed 1 (default) should fail
    let res = superfeed_api::delete_superfeed(db, 1).await;
    assert!(res.is_err());

    superfeed_api::delete_superfeed(db, second_superfeed_id).await?;
    let res = superfeed_api::get_superfeed(db, second_superfeed_id).await;
    assert!(matches!(res, Err(DbErr::RecordNotFound(_))));

    Ok(())
}

async fn article_api_test(db: &DbConn) -> Result<(), DbErr> {
    let published = Utc::now() - chrono::TimeDelta::days(2);
    let empty_related_articles = feed_api::get_articles(db, 1, ReadFilter::All, None, None).await?;
    assert_eq!(empty_related_articles.len(), 0);

    article_api::create_article(
        db,
        article_api::article_max_id(db).await? + 1,
        "https://kottke.org/25/02/the-sutro-tower-in-3d".to_owned(),
        "The Sutro Tower in 3D".to_owned(),
        published,
        published + chrono::TimeDelta::days(3), // lifespan 3 days, relative age (now - (now-2)) / 3 = 2/3 = 0.66
        false,
        "This is an amazingly realistic 3D model of San Francisco's Sutro 
            Tower that you can zoom, pan, fly through, and interact with."
            .to_owned(),
        1,
    )
    .await?;

    let next_published = Utc::now() - chrono::TimeDelta::days(1);
    article_api::create_article(
        db,
        article_api::article_max_id(db).await? + 1,
        "https://kottke.org/25/03/dont-be-a-sucker".to_owned(),
        "Don't Be A Sucker".to_owned(),
        next_published,
        next_published + chrono::TimeDelta::days(3), // lifespan 3 days, relative age (now - (now-1)) / 3 = 1/3 = 0.33
        false,
        "In 1945, the US Department of War (the precursor to the Dept of 
            Defense) produced this educational film on the destructive effects 
            of racial and religious prejudice and the use of such prejudice to gain 
            power."
            .to_owned(),
        1,
    )
    .await?;

    let new_related_articles = feed_api::get_articles(db, 1, ReadFilter::All, None, None).await?;
    assert_eq!(new_related_articles.len(), 2);
    assert_eq!(new_related_articles[0].name, "Don't Be A Sucker".to_owned());

    let article1 = article_api::get_article(db, 1).await?;
    assert_eq!(article1.name, "The Sutro Tower in 3D".to_owned());
    assert_eq!(article1.read, false);

    let article1_by_url = article_api::get_article_by_url(
        db,
        "https://kottke.org/25/02/the-sutro-tower-in-3d".to_owned(),
    )
    .await?;
    assert_eq!(article1_by_url.unwrap().id, 1);

    let no_article = article_api::get_article_by_url(db, "fake_url".to_owned()).await?;
    assert!(no_article.is_none());

    article_api::read_article(db, 1).await?;
    let article1_read = article_api::get_article(db, 1).await?;
    assert_eq!(article1_read.read, true);

    let article2 = article_api::get_article(db, 2).await?;
    assert_eq!(article2.read, false);
    article_api::read_all(db, 1).await?;
    let article2_read = article_api::get_article(db, 2).await?;
    assert_eq!(article2_read.read, true);

    Ok(())
}

async fn tag_api_test(db: &DbConn) -> Result<(), DbErr> {
    let all_tags = tag_api::get_all_tags(db).await?;
    assert_eq!(all_tags.len(), 0);

    let _ = tag_api::create_tag(
        db,
        tag_api::tag_max_id(db).await? + 1,
        "Cool Stuff".to_owned(),
    )
    .await;
    let _ = tag_api::create_tag(
        db,
        tag_api::tag_max_id(db).await? + 1,
        "Read Later".to_owned(),
    )
    .await;
    let _ = tag_api::tag_article(db, 1, 1).await;
    let _ = tag_api::tag_article(db, 2, 1).await;
    let _ = tag_api::tag_article(db, 1, 2).await;

    let all_tags = tag_api::get_all_tags(db).await?;
    assert_eq!(all_tags.len(), 2);

    // Backslash restriction test
    let backslash_err = tag_api::create_tag(db, 3, "bad\\tag".to_owned()).await;
    assert!(backslash_err.is_err());
    let rename_err = tag_api::rename_tag(db, 1, "bad\\tag".to_owned()).await;
    assert!(rename_err.is_err());

    tag_api::rename_tag(db, 2, "Read Later".to_owned()).await?;
    let renamed_tag = tag_api::get_tag(db, 2).await?;
    assert_eq!(renamed_tag.name, "Read Later");

    let cool_articles = tag_api::get_articles(db, 1, ReadFilter::All, None, None).await?;
    assert_eq!(cool_articles.len(), 2);

    let sutro_tags = article_api::get_tags(db, 1).await?;
    assert_eq!(sutro_tags.len(), 2);

    tag_api::delete_tag(db, 2).await?;
    let new_sutro_tags = article_api::get_tags(db, 1).await?;
    assert_eq!(new_sutro_tags.len(), 1);

    tag_api::delete_tag_article(db, 1, 1).await?;
    let newest_sutro_tags = article_api::get_tags(db, 1).await?;
    assert_eq!(newest_sutro_tags.len(), 0);

    Ok(())
}
