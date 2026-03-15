use super::{unwrap_date, unwrap_default, RetrievalError};
use crate::api::{article_api, feed_api};
use crate::models::feed;
use article_api::get_article_by_url;
use atom_syndication::Feed;
use chrono::{DateTime, Utc};
use reqwest::Client;
use rss::validation::Validate;
use rss::Channel;
use sea_orm::DbConn;
use sea_orm::DbErr;
use std::time::Duration;

const USER_AGENT: &str = "Celadon/0.0.1 (https://github.com/aahaansingh/Celadon)";
const MAX_RETRY_AFTER_SECS: u64 = 24 * 3600; // 24 hours
const FAR_FUTURE_DAYS: i64 = 365;
/// Minimum interval between polls per feed (rachelbythebay and similar: "at most once per hour").
const MIN_POLL_INTERVAL_HOURS: i64 = 1;
/// Extra seconds added on top of 429 Retry-After to give servers leeway
const EXTRA_DELAY_429_SECS: i64 = 60;

/// Result of a conditional feed fetch.
pub enum FetchResult {
    /// 200 OK: body, optional cache headers, and final URL if redirect occurred
    Full(
        bytes::Bytes,
        Option<String>,
        Option<String>,
        Option<String>, // final_url when different from request url (redirect)
    ),
    /// 304 Not Modified: no body
    NotModified,
}

/// Error from feed fetch (rate limit, HTTP error, or network).
#[derive(Debug)]
pub enum FetchError {
    RateLimited { retry_after: Option<DateTime<Utc>> },
    Http {
        code: u16,
        retry_after: Option<DateTime<Utc>>, // for 4xx/5xx, optional Retry-After
    },
    Network(Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::RateLimited { retry_after } => {
                write!(f, "Rate limited")?;
                if let Some(until) = retry_after {
                    write!(f, " (retry after {})", until)?;
                }
                Ok(())
            }
            FetchError::Http { code, .. } => write!(f, "HTTP error {}", code),
            FetchError::Network(e) => write!(f, "Network error: {}", e),
        }
    }
}

impl std::error::Error for FetchError {}

/// Build a shared HTTP client with User-Agent and gzip support.
fn http_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .build()
}

/// Parse Retry-After header: delta-seconds (integer) or HTTP-date. Returns None if missing or unparseable.
fn parse_retry_after(value: &str) -> Option<DateTime<Utc>> {
    let value = value.trim();
    // Try delta-seconds first
    if let Ok(secs) = value.parse::<u64>() {
        let secs = secs.min(MAX_RETRY_AFTER_SECS);
        return Some(Utc::now() + chrono::TimeDelta::seconds(secs as i64));
    }
    // Try HTTP-date (RFC 7231)
    if let Ok(dt) = DateTime::parse_from_rfc2822(value) {
        let utc = dt.to_utc();
        let max_after = Utc::now() + chrono::TimeDelta::seconds(MAX_RETRY_AFTER_SECS as i64);
        return Some(if utc > max_after { max_after } else { utc });
    }
    None
}

/// Fetch feed bytes from URL with optional conditional headers. Uses a single reqwest Client.
pub async fn fetch_feed_bytes(
    url: &str,
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> Result<FetchResult, FetchError> {
    let client = http_client().map_err(|e| FetchError::Network(Box::new(e)))?;
    let mut request = client.get(url);
    if let Some(e) = etag {
        request = request.header("If-None-Match", e);
    }
    if let Some(lm) = last_modified {
        request = request.header("If-Modified-Since", lm);
    }
    let response = request
        .send()
        .await
        .map_err(|e| FetchError::Network(Box::new(e)))?;
    let status = response.status();
    match status.as_u16() {
        200 => {
            let etag = response
                .headers()
                .get("ETag")
                .and_then(|v| v.to_str().ok())
                .map(String::from);
            let last_modified = response
                .headers()
                .get("Last-Modified")
                .and_then(|v| v.to_str().ok())
                .map(String::from);
            let final_url = {
                let final_u = response.url().to_string();
                if final_u != url {
                    Some(final_u)
                } else {
                    None
                }
            };
            let body = response
                .bytes()
                .await
                .map_err(|e| FetchError::Network(Box::new(e)))?;
            Ok(FetchResult::Full(body, etag, last_modified, final_url))
        }
        304 => Ok(FetchResult::NotModified),
        429 => {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(parse_retry_after);
            Err(FetchError::RateLimited { retry_after })
        }
        code if (400..=599).contains(&code) => {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(parse_retry_after);
            Err(FetchError::Http {
                code,
                retry_after,
            })
        }
        _ => Err(FetchError::Http {
            code: status.as_u16(),
            retry_after: None,
        }),
    }
}

pub enum SyndicationFeed {
    Rss(Channel),
    Atom(Feed),
    Json(JsonFeed),
}

/// Minimal JSON Feed 1.1 struct for parsing (https://jsonfeed.org/version/1.1/).
#[derive(Debug, serde::Deserialize)]
pub struct JsonFeed {
    pub title: Option<String>,
    #[serde(default)]
    pub items: Vec<JsonFeedItem>,
}

#[derive(Debug, serde::Deserialize)]
pub struct JsonFeedItem {
    pub id: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "date_published")]
    pub date_published: Option<String>,
    #[serde(rename = "date_modified")]
    pub date_modified: Option<String>,
    #[serde(rename = "content_html")]
    pub content_html: Option<String>,
    #[serde(rename = "content_text")]
    pub content_text: Option<String>,
}

/// Parse response body into JSON Feed, RSS, or Atom (tries JSON first if body looks like JSON).
fn bytes_to_syndication_feed(body: &[u8]) -> Result<SyndicationFeed, Box<dyn std::error::Error>> {
    if body.first() == Some(&b'{') {
        if let Ok(json) = serde_json::from_slice::<JsonFeed>(body) {
            if !json.items.is_empty() || json.title.is_some() {
                return Ok(SyndicationFeed::Json(json));
            }
        }
    }
    match atom_syndication::Feed::read_from(body) {
        Ok(obj) => Ok(SyndicationFeed::Atom(obj)),
        Err(_) => match rss::Channel::read_from(body) {
            Ok(obj) => Ok(SyndicationFeed::Rss(obj)),
            Err(_) => Err(RetrievalError.into()),
        },
    }
}

// Legacy: fetch URL with no conditional headers and parse. Used only when we need to fetch without a feed model (e.g. callers that don't have etag/last_modified).
pub async fn url_to_obj(url: &str) -> Result<SyndicationFeed, Box<dyn std::error::Error>> {
    let result = fetch_feed_bytes(url, None, None).await.map_err(Box::<dyn std::error::Error>::from)?;
    match result {
        FetchResult::Full(body, _, _, _) => bytes_to_syndication_feed(&body),
        FetchResult::NotModified => Err("Server returned 304 Not Modified".into()),
    }
}

pub async fn url_to_feed(
    db: &DbConn,
    url: String,
    superfeed_id: i32,
    feed_type: feed::FeedType,
) -> Result<(), Box<dyn std::error::Error>> {
    let matching_feeds = feed_api::get_feed_by_url(db, url.clone()).await?;
    match matching_feeds {
        None => {
            // New feed: fetch without conditionals; reject on 304/429/4xx/5xx or non-XML
            let result = fetch_feed_bytes(&url, None, None).await.map_err(Box::<dyn std::error::Error>::from)?;
            match result {
                FetchResult::Full(body, etag, last_modified, final_url) => {
                    let feed_url = final_url.unwrap_or(url);
                    let feed_obj = bytes_to_syndication_feed(&body)?;
                    new_feed(db, feed_obj, feed_url.clone(), superfeed_id, feed_type).await?;
                    let feed_id = feed_api::get_feed_by_url(db, feed_url).await?.expect("just created").id;
                    feed_api::update_feed_conditional_headers(db, feed_id, etag, last_modified).await?;
                    feed_api::update_feed_next_poll_after(
                        db,
                        feed_id,
                        Some(Utc::now() + chrono::TimeDelta::hours(MIN_POLL_INTERVAL_HOURS)),
                    )
                    .await?;
                    Ok(())
                }
                FetchResult::NotModified => Err("Server returned 304 for new feed".into()),
            }
        }
        Some(matched_feed) => {
            let result = fetch_feed_bytes(
                &url,
                matched_feed.etag.as_deref(),
                matched_feed.last_modified.as_deref(),
            )
            .await;
            match result {
                Ok(FetchResult::Full(body, etag, last_modified, final_url)) => {
                    if let Some(ref final_u) = final_url {
                        if final_u != &url {
                            // Redirect: check if another feed already has this URL before we update
                            if let Ok(Some(existing)) = feed_api::get_feed_by_url(db, final_u.clone()).await {
                                if existing.id != matched_feed.id {
                                    // Redirect target is another feed; mark this feed dead
                                    feed_api::update_feed_consecutive_http_errors(db, matched_feed.id, 3).await?;
                                    let far_future = Utc::now() + chrono::TimeDelta::days(FAR_FUTURE_DAYS);
                                    feed_api::update_feed_next_poll_after(db, matched_feed.id, Some(far_future))
                                        .await?;
                                    return Ok(());
                                }
                            }
                            feed_api::update_feed_url(db, matched_feed.id, final_u.clone()).await?;
                        }
                    }
                    feed_api::update_feed_consecutive_http_errors(db, matched_feed.id, 0).await?;
                    let feed_obj = bytes_to_syndication_feed(&body)?;
                    update_feed(db, matched_feed.id, feed_obj).await?;
                    feed_api::update_feed_status(db, matched_feed.id, 0).await?;
                    // Compliant: poll at most once per hour (rachelbythebay et al.).
                    feed_api::update_feed_next_poll_after(
                        db,
                        matched_feed.id,
                        Some(Utc::now() + chrono::TimeDelta::hours(MIN_POLL_INTERVAL_HOURS)),
                    )
                    .await?;
                    feed_api::update_feed_conditional_headers(
                        db,
                        matched_feed.id,
                        etag,
                        last_modified,
                    )
                    .await?;
                    Ok(())
                }
                Ok(FetchResult::NotModified) => {
                    feed_api::update_feed_consecutive_http_errors(db, matched_feed.id, 0).await?;
                    feed_api::update_feed_dt(
                        db,
                        matched_feed.id,
                        feed_api::FeedDtFields::LastFetched,
                        Utc::now(),
                    )
                    .await?;
                    feed_api::update_feed_status(db, matched_feed.id, 0).await?;
                    feed_api::update_feed_next_poll_after(
                        db,
                        matched_feed.id,
                        Some(Utc::now() + chrono::TimeDelta::hours(MIN_POLL_INTERVAL_HOURS)),
                    )
                    .await?;
                    Ok(())
                }
                Err(FetchError::RateLimited { retry_after }) => {
                    feed_api::update_feed_status(db, matched_feed.id, 1).await?;
                    let until = retry_after
                        .unwrap_or_else(|| Utc::now() + chrono::TimeDelta::hours(1));
                    let until = until + chrono::TimeDelta::seconds(EXTRA_DELAY_429_SECS);
                    feed_api::update_feed_next_poll_after(db, matched_feed.id, Some(until)).await?;
                    Err(FetchError::RateLimited { retry_after }.into())
                }
                Err(FetchError::Http { code, retry_after }) => {
                    let feed = feed_api::get_feed(db, matched_feed.id).await?;
                    let n = feed.consecutive_http_errors;
                    let now = Utc::now();
                    if n == 0 {
                        let next = retry_after
                            .unwrap_or_else(|| now + chrono::TimeDelta::days(1));
                        feed_api::update_feed_status(db, matched_feed.id, code as i32).await?;
                        feed_api::update_feed_next_poll_after(db, matched_feed.id, Some(next)).await?;
                        feed_api::update_feed_consecutive_http_errors(db, matched_feed.id, 1).await?;
                    } else if n == 1 {
                        let next = now + chrono::TimeDelta::days(7);
                        feed_api::update_feed_status(db, matched_feed.id, code as i32).await?;
                        feed_api::update_feed_next_poll_after(db, matched_feed.id, Some(next)).await?;
                        feed_api::update_feed_consecutive_http_errors(db, matched_feed.id, 2).await?;
                    } else {
                        // Third failure: mark dead, stop polling; keep status as most recent error code
                        feed_api::update_feed_status(db, matched_feed.id, code as i32).await?;
                        feed_api::update_feed_consecutive_http_errors(db, matched_feed.id, 3).await?;
                        let far_future = now + chrono::TimeDelta::days(FAR_FUTURE_DAYS);
                        feed_api::update_feed_next_poll_after(db, matched_feed.id, Some(far_future))
                            .await?;
                    }
                    Err(FetchError::Http {
                        code,
                        retry_after,
                    }
                    .into())
                }
                Err(FetchError::Network(e)) => Err(FetchError::Network(e).into()),
            }
        }
    }
}

fn parse_json_feed_date(s: Option<&String>) -> DateTime<Utc> {
    match s.and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()) {
        Some(dt) => dt.with_timezone(&Utc),
        None => Utc::now(),
    }
}

pub fn calculate_expiry(published: DateTime<Utc>, feed_type: &feed::FeedType) -> DateTime<Utc> {
    let duration = match feed_type {
        feed::FeedType::News => chrono::TimeDelta::days(1),
        feed::FeedType::Article => chrono::TimeDelta::days(3),
        feed::FeedType::Essay => chrono::TimeDelta::days(7),
        feed::FeedType::Update => chrono::TimeDelta::hours(6),
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
                0,
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
                0,
                feed_type.clone(),
            )
            .await?;
        }
        SyndicationFeed::Json(ref json_feed) => {
            feed_api::create_feed(
                db,
                feed_id,
                url,
                json_feed.title.clone().unwrap_or_else(|| "Untitled".to_owned()),
                "".to_owned(),
                Utc::now(),
                Utc::now(),
                0,
                feed_type.clone(),
            )
            .await?;
        }
    }

    // Add to superfeed
    feed_api::add_feed_to_superfeed(db, feed_id, superfeed_id).await?;

    // Add articles (mark as read if already past expiry — backloaded articles)
    match feed {
        SyndicationFeed::Rss(ref channel) => {
            for article in channel.items.iter() {
                let published = unwrap_date(article.pub_date.clone());
                let expiry_at = calculate_expiry(published, &feed_type);
                let already_expired = Utc::now() >= expiry_at;
                // Prefer content:encoded (full article HTML) over description (often just a summary); Substack and others use content:encoded
                let description = article
                    .content
                    .clone()
                    .filter(|s| !s.trim().is_empty())
                    .or_else(|| article.description.clone())
                    .unwrap_or_else(|| "No description provided.".to_owned());
                article_api::create_article(
                    db,
                    article_api::article_max_id(db).await? + 1,
                    unwrap_default(article.link.clone(), channel.link.clone()),
                    unwrap_default(article.title.clone(), channel.title.clone()),
                    published,
                    expiry_at,
                    already_expired,
                    description,
                    feed_id,
                )
                .await?;
            }
        }
        SyndicationFeed::Atom(ref feed_inner) => {
            for article in feed_inner.entries.iter() {
                // Atom: published is optional; fall back to updated (required) so feeds that omit published still get real dates
                let published = unwrap_default(article.published, article.updated.clone()).to_utc();
                let expiry_at = calculate_expiry(published, &feed_type);
                let already_expired = Utc::now() >= expiry_at;
                // Substack and some Atom feeds use only <summary> or have empty <content>; fall back to summary
                let default_desc = article
                    .summary
                    .as_ref()
                    .map(|t| t.value.clone())
                    .unwrap_or_else(|| "No description provided.".to_owned());
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
                    expiry_at,
                    already_expired,
                    unwrap_atom_content(article.content.clone(), default_desc),
                    feed_id,
                )
                .await?;
            }
        }
        SyndicationFeed::Json(ref json_feed) => {
            let feed_title = json_feed.title.clone().unwrap_or_else(|| "Untitled".to_owned());
            for item in json_feed.items.iter() {
                let published = parse_json_feed_date(
                    item.date_published.as_ref().or(item.date_modified.as_ref()),
                );
                let expiry_at = calculate_expiry(published, &feed_type);
                let already_expired = Utc::now() >= expiry_at;
                let article_url = item
                    .url
                    .clone()
                    .or_else(|| item.id.clone())
                    .unwrap_or_default();
                let description = item
                    .content_html
                    .clone()
                    .or_else(|| item.content_text.clone())
                    .unwrap_or_else(|| "No description provided.".to_owned());
                article_api::create_article(
                    db,
                    article_api::article_max_id(db).await? + 1,
                    article_url,
                    item.title.clone().unwrap_or_else(|| feed_title.clone()),
                    published,
                    expiry_at,
                    already_expired,
                    description,
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
                feed_api::update_feed_status(db, id, 500).await?;
                let far_future = Utc::now() + chrono::TimeDelta::days(365);
                feed_api::update_feed_next_poll_after(db, id, Some(far_future)).await?;
                return Ok(());
            }
            Ok(_) => {
                for article in channel.items.iter() {
                    let article_url = unwrap_default(article.link.clone(), channel.link.clone());
                    match get_article_by_url(db, article_url.clone()).await? {
                        None => {
                            let published = unwrap_date(article.pub_date.clone());
                            let expiry_at = calculate_expiry(published, &feed_model.feed_type);
                            let already_expired = Utc::now() >= expiry_at;
                            // Prefer content:encoded (full article HTML) over description; Substack uses content:encoded
                            let description = article
                                .content
                                .clone()
                                .filter(|s| !s.trim().is_empty())
                                .or_else(|| article.description.clone())
                                .unwrap_or_else(|| "No description provided.".to_owned());
                            article_api::create_article(
                                db,
                                article_api::article_max_id(db).await? + 1,
                                article_url,
                                unwrap_default(article.title.clone(), channel.title.clone()),
                                published,
                                expiry_at,
                                already_expired,
                                description,
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
                        // Atom: published is optional; fall back to updated so feeds that omit published get real dates
                        let published = unwrap_default(article.published, article.updated.clone())
                            .to_utc();
                        let expiry_at = calculate_expiry(published, &feed_model.feed_type);
                        let already_expired = Utc::now() >= expiry_at;
                        // Substack and some Atom feeds use only <summary> or have empty <content>; fall back to summary
                        let default_desc = article
                            .summary
                            .as_ref()
                            .map(|t| t.value.clone())
                            .unwrap_or_else(|| "No description provided.".to_owned());
                        article_api::create_article(
                            db,
                            article_api::article_max_id(db).await? + 1,
                            article_url,
                            article.title.value.clone(),
                            published,
                            expiry_at,
                            already_expired,
                            unwrap_atom_content(article.content.clone(), default_desc),
                            id,
                        )
                        .await?;
                    }
                    Some(_) => {}
                }
            }
        }
        SyndicationFeed::Json(ref json_feed) => {
            let feed_title = json_feed.title.clone().unwrap_or_else(|| "Untitled".to_owned());
            for item in json_feed.items.iter() {
                let article_url = item
                    .url
                    .clone()
                    .or_else(|| item.id.clone())
                    .unwrap_or_default();
                match get_article_by_url(db, article_url.clone()).await? {
                    None => {
                        let published = parse_json_feed_date(
                            item.date_published.as_ref().or(item.date_modified.as_ref()),
                        );
                        let expiry_at = calculate_expiry(published, &feed_model.feed_type);
                        let already_expired = Utc::now() >= expiry_at;
                        let description = item
                            .content_html
                            .clone()
                            .or_else(|| item.content_text.clone())
                            .unwrap_or_else(|| "No description provided.".to_owned());
                        article_api::create_article(
                            db,
                            article_api::article_max_id(db).await? + 1,
                            article_url,
                            item.title
                                .clone()
                                .unwrap_or_else(|| feed_title.clone()),
                            published,
                            expiry_at,
                            already_expired,
                            description,
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

/// Re-fetch all feeds from their URLs and insert any new articles. Used by the hourly background
/// task; one failing feed does not stop the rest. Skips feeds when next_poll_after is in the future or consecutive_http_errors >= 3 (dead).
pub async fn refresh_all_feeds(db: &DbConn) -> Result<(), DbErr> {
    let feeds = feed_api::get_all_feeds(db).await?;
    const ALL_SUPERFEED_ID: i32 = 1;
    let now = Utc::now();
    for f in feeds {
        let skip_by_time = f
            .next_poll_after
            .map(|t| now < t)
            .unwrap_or(false);
        let skip_broken = f.consecutive_http_errors >= 3;
        if skip_by_time || skip_broken {
            continue;
        }
        if let Err(e) = url_to_feed(db, f.url.clone(), ALL_SUPERFEED_ID, f.feed_type.clone()).await
        {
            eprintln!("refresh_all_feeds: feed id {} ({}) failed: {}", f.id, f.url, e);
        }
    }
    let _ = article_api::ensure_article_cap(db, article_api::ARTICLE_CAP).await;
    Ok(())
}

/// Re-fetch the given feeds by id (fetch from URL and insert articles). Used after OPML import.
/// One failing feed does not stop the rest. Skips feeds when next_poll_after is in the future or consecutive_http_errors >= 3 (dead).
pub async fn refresh_feeds_by_ids(db: &DbConn, feed_ids: Vec<i32>) -> Result<(), DbErr> {
    const ALL_SUPERFEED_ID: i32 = 1;
    let now = Utc::now();
    for id in feed_ids {
        let feed = match feed_api::get_feed(db, id).await {
            Ok(f) => f,
            Err(_) => continue,
        };
        let skip_by_time = feed
            .next_poll_after
            .map(|t| now < t)
            .unwrap_or(false);
        let skip_broken = feed.consecutive_http_errors >= 3;
        if skip_by_time || skip_broken {
            continue;
        }
        if let Err(e) =
            url_to_feed(db, feed.url.clone(), ALL_SUPERFEED_ID, feed.feed_type.clone()).await
        {
            eprintln!("refresh_feeds_by_ids: feed id {} ({}) failed: {}", id, feed.url, e);
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
