use super::{unwrap_date, unwrap_default, RetrievalError};
use crate::api::*;
use article_api::get_article_by_url;
use chrono::{DateTime, Utc};
use reqwest;
use rss::validation::Validate;
use rss::Channel;
use sea_orm::{
    entity::*, error::*, query::*, sea_query, tests_cfg::*, Database, DbConn, DeleteResult,
};
use std::fs::File;
use std::io::BufReader;

pub enum FeedType {
    Rss,
    Atom,
}
// TODO make a function to get the feed url from site url
pub async fn url_to_feed(db: &DbConn, url: String, folder: i32) -> Result<(), Box<dyn std::error::Error>> {
    let content = reqwest::get(url.clone()).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    let matching_feeds = feed_api::get_feed_by_url(db, url.clone()).await?;
    match matching_feeds {
        None => {
            let new_feed_res = new_feed(db, channel, url, folder).await?;
            Ok(new_feed_res)
        }
        Some(feed) => {
            let update_feed_res = update_feed(db, feed.id, channel).await?;
            Ok(update_feed_res)
        }
    }
}

pub async fn new_feed(db: &DbConn, channel: Channel, url: String, folder: i32) -> Result<(), Box<dyn std::error::Error>> {
    let feed_id = feed_api::feed_max_id(db).await? + 1;
    feed_api::create_feed(
        db,
        feed_id,
        url,
        channel.title.clone(),
        "".to_owned(), // I didn't realize that a feed could have multiple categories, blank for now
        Utc::now(),
        Utc::now(),
        true,
        folder,
    )
    .await;

    // Should be ok to clone
    for article in channel.items.iter() {
        article_api::create_article(
            db,
            article_api::article_max_id(db).await? + 1,
            unwrap_default(article.link.clone(), channel.link.clone()),
            unwrap_default(article.title.clone(), channel.title.clone()),
            unwrap_date(article.pub_date.clone()),
            false,
            unwrap_default(
                article.description.clone(),
                "No description provided.".to_owned(),
            ),
            feed_id,
        )
        .await;
    }
    Ok(())
}

pub async fn update_feed(
    db: &DbConn,
    id: i32,
    channel: Channel
) -> Result<(), Box<dyn std::error::Error>> {
    feed_api::update_feed_dt(db, id, feed_api::FeedDtFields::LastFetched, Utc::now()).await?;
    match channel.validate() {
        Err(e) => {
            feed_api::update_feed_health(db, id, false).await?;
            return Ok(());
        }
        Ok(_) => {
            for article in channel.items.iter() {
                // For now, we are uniquely identifying articles by URL even though for
                // broken feeds this might not entirely suffice
                match get_article_by_url(db, channel.link.clone()).await? {
                    None => {
                        article_api::create_article(
                            db,
                            article_api::article_max_id(db).await? + 1,
                            unwrap_default(article.link.clone(), channel.link.clone()),
                            unwrap_default(article.title.clone(), channel.title.clone()),
                            unwrap_date(article.pub_date.clone()),
                            false,
                            unwrap_default(
                                article.description.clone(),
                                "No description provided.".to_owned(),
                            ),
                            id,
                        )
                        .await;
                    }
                    Some(_) => {}
                }
            }
        }
    }
    Ok(())
}
