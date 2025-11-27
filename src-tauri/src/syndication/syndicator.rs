use super::{unwrap_date, unwrap_default, RetrievalError};
use crate::api::*;
use article_api::get_article_by_url;
use atom_syndication::Feed;
use chrono::{DateTime, FixedOffset, Local, Utc};
use reqwest;
use rss::validation::Validate;
use rss::Channel;
use sea_orm::{
    entity::*, error::*, query::*, sea_query, tests_cfg::*, Database, DbConn, DeleteResult,
};
use std::fs::File;
use std::io::BufReader;

pub enum FeedType {
    Rss(Channel),
    Atom(Feed),
}

// Using the rust-syndication wrapper's method here...
pub async fn url_to_obj(url: &String) -> Result<FeedType, Box<dyn std::error::Error>> {
    let content = reqwest::get(url).await?.bytes().await?;
    match atom_syndication::Feed::read_from(&content[..]) {
        Ok(obj) => Ok(FeedType::Atom(obj)),
        Err(_) => match rss::Channel::read_from(&content[..]) {
            Ok(obj) => Ok(FeedType::Rss(obj)),
            Err(_) => Err(RetrievalError.into()),
        },
    }
}

// TODO make a function to get the feed url from site url
pub async fn url_to_feed(
    db: &DbConn,
    url: String,
    folder: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let feed_obj = url_to_obj(&url).await?;
    let matching_feeds = feed_api::get_feed_by_url(db, url.clone()).await?;
    match matching_feeds {
        None => {
            let new_feed_res = new_feed(db, feed_obj, url, folder).await?;
            Ok(new_feed_res)
        }
        Some(matched_feed) => {
            let update_feed_res = update_feed(db, matched_feed.id, feed_obj).await?;
            Ok(update_feed_res)
        }
    }
}

pub async fn new_feed(
    db: &DbConn,
    feed: FeedType,
    url: String,
    folder: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let feed_id = feed_api::feed_max_id(db).await? + 1;
    match feed {
        FeedType::Rss(channel) => {
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
            .await?;

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
                .await?;
            }
        }
        FeedType::Atom(feed) => {
            feed_api::create_feed(
                db,
                feed_id,
                url,
                feed.title.value.clone(),
                "".to_owned(), // I didn't realize that a feed could have multiple categories, blank for now
                Utc::now(),
                Utc::now(),
                true,
                folder,
            )
            .await?;

            for article in feed.entries.iter() {
                article_api::create_article(
                    db,
                    article_api::article_max_id(db).await? + 1,
                    article
                        .links
                        .get(0)
                        .map(|l| l.href.clone())
                        .unwrap_or_default(), // Should be a better url
                    article.title.value.clone(),
                    unwrap_default(article.published, Utc::now().into()).to_utc(),
                    false,
                    unwrap_atom_content(
                        article.content.clone(),
                        "No description provided.".to_owned(),
                    ),
                    feed_id,
                )
                .await?;
            }
        }
    }
    Ok(())
}

pub async fn update_feed(
    db: &DbConn,
    id: i32,
    feed: FeedType,
) -> Result<(), Box<dyn std::error::Error>> {
    feed_api::update_feed_dt(db, id, feed_api::FeedDtFields::LastFetched, Utc::now()).await?;
    match feed {
        FeedType::Rss(channel) => {
            match channel.validate() {
                Err(e) => {
                    feed_api::update_feed_health(db, id, false).await?;
                    return Ok(());
                }
                Ok(_) => {
                    for article in channel.items.iter() {
                        let article_url =
                            unwrap_default(article.link.clone(), channel.link.clone());
                        // For now, we are uniquely identifying articles by URL even though for
                        // broken feeds this might not entirely suffice
                        match get_article_by_url(db, article_url.clone()).await? {
                            None => {
                                article_api::create_article(
                                    db,
                                    article_api::article_max_id(db).await? + 1,
                                    article_url,
                                    unwrap_default(article.title.clone(), channel.title.clone()),
                                    unwrap_date(article.pub_date.clone()),
                                    false,
                                    unwrap_default(
                                        article.description.clone(),
                                        "No description provided.".to_owned(),
                                    ),
                                    id,
                                )
                                .await?;
                            }
                            Some(_) => {}
                        }
                    }
                }
            }
        }
        FeedType::Atom(feed) => {
            for article in feed.entries.iter() {
                let article_url = article
                    .links
                    .get(0)
                    .map(|l| l.href.clone())
                    .unwrap_or_default();
                match get_article_by_url(db, article_url.clone()).await? {
                    None => {
                        article_api::create_article(
                            db,
                            article_api::article_max_id(db).await? + 1,
                            article_url,
                            article.title.value.clone(),
                            unwrap_default(article.published, Utc::now().into()).to_utc(),
                            false,
                            unwrap_atom_content(
                                article.content.clone(),
                                "No description provided.".to_owned(),
                            ),
                            id,
                        )
                        .await?;
                    }
                    Some(_) => {}
                }
            }
        }
    }
    Ok(())
}

pub fn unwrap_atom_content(
    content_opt: Option<atom_syndication::Content>,
    default: String,
) -> String {
    match content_opt {
        None => default,
        Some(content) => match content.value {
            None => default,
            Some(val) => val,
        },
    }
}
