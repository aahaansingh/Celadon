use std::fs::File;
use std::io::BufReader;
use rss::Channel;
use chrono::{DateTime, Utc};
use sea_orm::{
    entity::*, error::*, query::*, sea_query, tests_cfg::*, Database, DbConn,
    DeleteResult,
};
use reqwest;
use super::{RetrievalError, unwrap_default, unwrap_date};
use crate::api::*;

pub enum FeedType {
    Rss,
    Atom
}

async fn url_to_feed(db: &DbConn, url: String) -> Result<(), Box<dyn std::error::Error>> {
    let content = reqwest::get(url.clone())
        .await?
        .bytes()
        .await?;
    let channel = Channel::read_from(&content[..])?;
    let matching_feeds = feed_api::get_feed_by_url(db, url).await?;
    match matching_feeds {
        None => {
            let new_feed_res = new_feed(db, channel).await?;
            Ok(new_feed_res)
        },
        Some(feed) => {
            let update_feed_res = update_feed(db, feed.id, channel).await?;
            Ok(update_feed_res)
        }
    }
}

async fn new_feed(db: &DbConn, channel: Channel) -> Result<(), Box<dyn std::error::Error>> {
    let feed_id = feed_api::feed_max_id(db).await? + 1;
    feed_api::create_feed(
        db,
        feed_id,
        channel.link.clone(),
        channel.title.clone(),
        "".to_owned(), // I didn't realize that a feed could have multiple categories, blank for now
        Utc::now(),
        Utc::now(),
        true,
        1
    ).await;
    
    // Should be ok to clone, 
    for article in channel.items.iter() {
        article_api::create_article(
            db,
            article_api::article_max_id(db).await? + 1,
            unwrap_default(article.link.clone(), channel.link.clone()),
            unwrap_default(article.title.clone(), channel.title.clone()),
            unwrap_date(article.pub_date.clone()),
            false,
            unwrap_default(article.description.clone(), "No description provided.".to_owned()),
            feed_id
        ).await;
    }
    Ok(())
}

async fn update_feed(db: &DbConn, id: i32, channel: Channel) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}