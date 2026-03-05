use super::utils::{self, TestDB};
use crate::models::*;
use chrono::Utc;
use sea_orm::{entity::*, error::*, query::*, sea_query, tests_cfg::*, Database, DbConn};

#[async_std::test]
async fn main() -> Result<(), DbErr> {
    let test_db = TestDB::new("sql_test").await;
    create_tables::create_tables(&test_db.db).await?;
    feed_test(&test_db.db).await?;

    Ok(())
}

async fn feed_test(db: &DbConn) -> Result<(), DbErr> {
    let add = Utc::now();
    let fetch = Utc::now();
    let expiry = add + chrono::TimeDelta::days(1);
    
    let root_fd = superfeed::ActiveModel {
        id: Set(1),
        name: Set("base".to_owned()),
        ..Default::default()
    };

    let _root_insert_suc = Superfeed::insert(root_fd)
        .exec(db)
        .await
        .expect("couldn't insert root superfeed");

    let osearch_feed = feed::ActiveModel {
        id: Set(1),
        url: Set("http://www.osearch.org/feed".to_owned()),
        name: Set("OSearch".to_owned()),
        category: Set("Science".to_owned()),
        added: Set(add),
        last_fetched: Set(fetch),
        healthy: Set(true),
        feed_type: Set(feed::FeedType::News),
        ..Default::default()
    };

    let _osearch_insert_suc = Feed::insert(osearch_feed)
        .exec(db)
        .await
        .expect("couldn't insert osearch feed");

    let kottke_feed = feed::ActiveModel {
        id: Set(2),
        url: Set("http://www.osearch.org/feed".to_owned()),
        name: Set("Kottke".to_owned()),
        category: Set("Blog".to_owned()),
        added: Set(add),
        last_fetched: Set(fetch),
        healthy: Set(true),
        feed_type: Set(feed::FeedType::Article),
        ..Default::default()
    };

    let _kottke_insert_suc = Feed::insert(kottke_feed)
        .exec(db)
        .await
        .expect("couldn't insert kottke feed");

    // Junction table entries
    let junction1 = feed_superfeed::ActiveModel {
        feed_id: Set(1),
        superfeed_id: Set(1),
    };
    FeedSuperfeed::insert(junction1).exec(db).await.expect("couldn't insert junction 1");

    let junction2 = feed_superfeed::ActiveModel {
        feed_id: Set(2),
        superfeed_id: Set(1),
    };
    FeedSuperfeed::insert(junction2).exec(db).await.expect("couldn't insert junction 2");

    let sutro_kottke_art = article::ActiveModel {
        id: Set(1),
        url: Set("https://kottke.org/25/02/the-sutro-tower-in-3d".to_owned()),
        name: Set("The Sutro Tower in 3D".to_owned()),
        published: Set(add),
        expiry_at: Set(expiry),
        read: Set(true),
        description: Set(
            "This is an amazingly realistic 3D model of San Francisco's Sutro 
            Tower that you can zoom, pan, fly through, and interact with."
                .to_owned(),
        ),
        feed: Set(2),
    };

    let _sutro_insert_suc = Article::insert(sutro_kottke_art)
        .exec(db)
        .await
        .expect("couldn't insert sutro article");

    let cool_tag = tag::ActiveModel {
        id: Set(1),
        name: Set("Cool Stuff".to_owned()),
    };

    let _cool_tag_insert_suc = Tag::insert(cool_tag)
        .exec(db)
        .await
        .expect("couldn't insert cool tag");

    let uncool_tag = tag::ActiveModel {
        id: Set(2),
        name: Set("Uncool Stuff".to_owned()),
    };

    let _uncool_tag_insert_suc = Tag::insert(uncool_tag)
        .exec(db)
        .await
        .expect("couldn't insert uncool tag");

    let sutro_tag = tag_article::ActiveModel {
        tag_id: Set(1),
        article_id: Set(1),
    };

    let _sutro_tag_insert_suc = TagArticle::insert(sutro_tag)
        .exec(db)
        .await
        .expect("couldn't insert sutro tag");

    Ok(())
}
