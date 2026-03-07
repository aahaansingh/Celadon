use super::{unwrap_date, unwrap_default, RetrievalError};
use crate::api::{article_api, feed_api};
use crate::models::feed;
use article_api::get_article_by_url;
use atom_syndication::Feed;
use chrono::{DateTime, Utc};
use reqwest;
use rss::validation::Validate;
use rss::Channel;
use sea_orm::DbConn;

pub enum SyndicationFeed {
    Rss(Channel),
    Atom(Feed),
}

// Using the rust-syndication wrapper's method here...
pub async fn url_to_obj(url: &String) -> Result<SyndicationFeed, Box<dyn std::error::Error>> {
    let content = reqwest::get(url).await?.bytes().await?;
    match atom_syndication::Feed::read_from(&content[..]) {
        Ok(obj) => Ok(SyndicationFeed::Atom(obj)),
        Err(_) => match rss::Channel::read_from(&content[..]) {
            Ok(obj) => Ok(SyndicationFeed::Rss(obj)),
            Err(_) => Err(RetrievalError.into()),
        },
    }
}

pub async fn url_to_feed(
    db: &DbConn,
    url: String,
    superfeed_id: i32,
    feed_type: feed::FeedType,
) -> Result<(), Box<dyn std::error::Error>> {
    let feed_obj = url_to_obj(&url).await?;
    let matching_feeds = feed_api::get_feed_by_url(db, url.clone()).await?;
    match matching_feeds {
        None => {
            let new_feed_res = new_feed(db, feed_obj, url, superfeed_id, feed_type).await?;
            Ok(new_feed_res)
        }
        Some(matched_feed) => {
            let update_feed_res = update_feed(db, matched_feed.id, feed_obj).await?;
            Ok(update_feed_res)
        }
    }
}

pub fn calculate_expiry(published: DateTime<Utc>, feed_type: &feed::FeedType) -> DateTime<Utc> {
    let duration = match feed_type {
        feed::FeedType::News => chrono::TimeDelta::days(1),
        feed::FeedType::Article => chrono::TimeDelta::days(3),
        feed::FeedType::Essay => chrono::TimeDelta::days(7),
    };
    published + duration
}

pub async fn new_feed(
    db: &DbConn,
    feed: SyndicationFeed,
    url: String,
    superfeed_id: i32,
    feed_type: feed::FeedType,
) -> Result<(), Box<dyn std::error::Error>> {
    let feed_id = feed_api::feed_max_id(db).await? + 1;
    match feed {
        SyndicationFeed::Rss(ref channel) => {
            feed_api::create_feed(
                db,
                feed_id,
                url,
                channel.title.clone(),
                "".to_owned(),
                Utc::now(),
                Utc::now(),
                true,
                feed_type.clone(),
            )
            .await?;
        }
        SyndicationFeed::Atom(ref feed_inner) => {
            feed_api::create_feed(
                db,
                feed_id,
                url,
                feed_inner.title.value.clone(),
                "".to_owned(),
                Utc::now(),
                Utc::now(),
                true,
                feed_type.clone(),
            )
            .await?;
        }
    }

    // Add to superfeed
    feed_api::add_feed_to_superfeed(db, feed_id, superfeed_id).await?;

    // Add articles
    match feed {
        SyndicationFeed::Rss(ref channel) => {
            for article in channel.items.iter() {
                let published = unwrap_date(article.pub_date.clone());
                article_api::create_article(
                    db,
                    article_api::article_max_id(db).await? + 1,
                    unwrap_default(article.link.clone(), channel.link.clone()),
                    unwrap_default(article.title.clone(), channel.title.clone()),
                    published,
                    calculate_expiry(published, &feed_type),
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
        SyndicationFeed::Atom(ref feed_inner) => {
            for article in feed_inner.entries.iter() {
                let published = unwrap_default(article.published, Utc::now().into()).to_utc();
                article_api::create_article(
                    db,
                    article_api::article_max_id(db).await? + 1,
                    article
                        .links
                        .get(0)
                        .map(|l| l.href.clone())
                        .unwrap_or_default(),
                    article.title.value.clone(),
                    published,
                    calculate_expiry(published, &feed_type),
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
    feed: SyndicationFeed,
) -> Result<(), Box<dyn std::error::Error>> {
    let feed_model = feed_api::get_feed(db, id).await?;
    feed_api::update_feed_dt(db, id, feed_api::FeedDtFields::LastFetched, Utc::now()).await?;
    match feed {
        SyndicationFeed::Rss(ref channel) => match channel.validate() {
            Err(_) => {
                feed_api::update_feed_health(db, id, false).await?;
                return Ok(());
            }
            Ok(_) => {
                for article in channel.items.iter() {
                    let article_url = unwrap_default(article.link.clone(), channel.link.clone());
                    match get_article_by_url(db, article_url.clone()).await? {
                        None => {
                            let published = unwrap_date(article.pub_date.clone());
                            article_api::create_article(
                                db,
                                article_api::article_max_id(db).await? + 1,
                                article_url,
                                unwrap_default(article.title.clone(), channel.title.clone()),
                                published,
                                calculate_expiry(published, &feed_model.feed_type),
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
        },
        SyndicationFeed::Atom(ref feed_inner) => {
            for article in feed_inner.entries.iter() {
                let article_url = article
                    .links
                    .get(0)
                    .map(|l| l.href.clone())
                    .unwrap_or_default();
                match get_article_by_url(db, article_url.clone()).await? {
                    None => {
                        let published =
                            unwrap_default(article.published, Utc::now().into()).to_utc();
                        article_api::create_article(
                            db,
                            article_api::article_max_id(db).await? + 1,
                            article_url,
                            feed_inner.title.value.clone(),
                            published,
                            calculate_expiry(published, &feed_model.feed_type),
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
