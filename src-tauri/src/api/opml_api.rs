use crate::api::{feed_api, superfeed_api};
use crate::models::feed::FeedType;
use chrono::Utc;
use opml::{Body, Head, Outline, OPML};
use sea_orm::DbConn;

/// Returns true if the URL is from a known reader's tag/stream/label system rather than a real RSS/Atom feed.
/// Such outlines should be skipped on import so tags are not created as feeds.
fn is_likely_reader_tag_or_stream_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    // Feedly: tag and board URLs (e.g. feedly.com/v3/tags/..., feedly.com/v3/boards/...)
    if lower.contains("feedly.com") && (lower.contains("/v3/tags") || lower.contains("/v3/boards"))
    {
        return true;
    }
    // Inoreader: stream/label/subscription API URLs (e.g. inoreader.com/reader/..., stream IDs)
    if lower.contains("inoreader.com")
        && (lower.contains("/reader/") || lower.contains("/stream/") || lower.contains("/label/"))
    {
        return true;
    }
    // The Old Reader: user-specific stream URLs
    if lower.contains("theoldreader.com")
        && (lower.contains("/stream/") || lower.contains("/label/"))
    {
        return true;
    }
    // Netvibes, Newsblur and similar: subscription/stream URLs that are not direct feed URLs
    if lower.contains("netvibes.com") && lower.contains("/subscribe/") {
        return true;
    }
    false
}

/// Returns true if this outline should be treated as a tag/category (not a feed) and skipped when importing feeds.
fn outline_is_tag_or_category(outline: &Outline) -> bool {
    if let Some(ref t) = outline.r#type {
        let lower = t.to_lowercase();
        if lower == "tag" || lower == "category" || lower == "label" {
            return true;
        }
    }
    false
}

/// Ensure feed is in superfeed; no-op if already linked.
async fn ensure_feed_in_superfeed(
    db: &DbConn,
    feed_id: i32,
    superfeed_id: i32,
) -> Result<(), String> {
    let ids = feed_api::get_superfeed_ids_for_feed(db, feed_id)
        .await
        .map_err(|e| e.to_string())?;
    if ids.contains(&superfeed_id) {
        return Ok(());
    }
    feed_api::add_feed_to_superfeed(db, feed_id, superfeed_id)
        .await
        .map_err(|e| e.to_string())
}

pub async fn import_opml_internal(db: &DbConn, path: String) -> Result<(), String> {
    let xml = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    import_opml_from_xml(db, xml).await
}

pub async fn import_opml_from_xml(db: &DbConn, xml: String) -> Result<(), String> {
    let xml = xml.trim();
    if xml.is_empty() {
        return Err("OPML content is empty".to_string());
    }
    let document = OPML::from_str(xml).map_err(|e| e.to_string())?;

    const MAX_OUTLINES: usize = 10_000;
    let total_outlines = document.body.outlines.len()
        + document
            .body
            .outlines
            .iter()
            .map(|o| o.outlines.len())
            .sum::<usize>();
    if total_outlines > MAX_OUTLINES {
        return Err(format!(
            "OPML has too many feeds ({}). Maximum is {}.",
            total_outlines, MAX_OUTLINES
        ));
    }

    let now = Utc::now();
    const YIELD_EVERY: usize = 30;
    let mut new_feed_ids = Vec::new();

    for (idx, outline) in document.body.outlines.iter().enumerate() {
        if idx > 0 && idx % YIELD_EVERY == 0 {
            tokio::task::yield_now().await;
        }
        if outline.xml_url.is_some() {
            let url = outline.xml_url.clone().unwrap_or_default();
            if outline_is_tag_or_category(outline) || is_likely_reader_tag_or_stream_url(&url) {
                continue;
            }
            let name = outline.text.clone();
            let feed_id = match feed_api::get_feed_by_url(db, url.clone())
                .await
                .map_err(|e| e.to_string())?
            {
                Some(existing) => existing.id,
                None => {
                    let feed_id = feed_api::feed_max_id(db).await.unwrap_or(0) + 1;
                    feed_api::create_feed(
                        db,
                        feed_id,
                        url,
                        name,
                        "Import".to_string(),
                        now,
                        now,
                        0,
                        FeedType::News,
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                    new_feed_ids.push(feed_id);
                    feed_id
                }
            };
            ensure_feed_in_superfeed(db, feed_id, 1).await?;
        } else {
            let superfeed_name = outline
                .title
                .clone()
                .unwrap_or_else(|| outline.text.clone());
            let sf_id = superfeed_api::superfeed_max_id(db).await.unwrap_or(0) + 1;
            let _ = superfeed_api::create_superfeed(db, sf_id, superfeed_name.clone()).await;

            for sub_outline in &outline.outlines {
                if sub_outline.xml_url.is_some() {
                    let url = sub_outline.xml_url.clone().unwrap_or_default();
                    if outline_is_tag_or_category(sub_outline)
                        || is_likely_reader_tag_or_stream_url(&url)
                    {
                        continue;
                    }
                    let name = sub_outline.text.clone();
                    let feed_id = match feed_api::get_feed_by_url(db, url.clone())
                        .await
                        .map_err(|e| e.to_string())?
                    {
                        Some(existing) => existing.id,
                        None => {
                            let feed_id = feed_api::feed_max_id(db).await.unwrap_or(0) + 1;
                            feed_api::create_feed(
                                db,
                                feed_id,
                                url,
                                name,
                                "Import".to_string(),
                                now,
                                now,
                                0,
                                FeedType::News,
                            )
                            .await
                            .map_err(|e| e.to_string())?;
                            new_feed_ids.push(feed_id);
                            feed_id
                        }
                    };
                    ensure_feed_in_superfeed(db, feed_id, sf_id).await?;
                }
            }
        }
    }

    // Fetch recent articles for newly imported feeds.
    if !new_feed_ids.is_empty() {
        if let Err(e) = crate::syndication::syndicator::refresh_feeds_by_ids(db, new_feed_ids).await
        {
            eprintln!(
                "OPML import: failed to fetch articles for some feeds: {}",
                e
            );
        }
    }

    Ok(())
}

pub async fn export_opml_internal(db: &DbConn, path: String) -> Result<(), String> {
    let feeds = feed_api::get_all_feeds(db)
        .await
        .map_err(|e| e.to_string())?;

    let outlines: Vec<Outline> = feeds
        .into_iter()
        .map(|f| Outline {
            text: f.name.clone(),
            title: Some(f.name.clone()),
            xml_url: Some(f.url.clone()),
            ..Default::default()
        })
        .collect();

    let opml = OPML {
        version: "2.0".to_string(),
        head: Some(Head {
            title: Some("Celadon OPML Export".to_string()),
            ..Default::default()
        }),
        body: Body { outlines },
    };

    let xml = opml.to_string().map_err(|e| e.to_string())?;
    std::fs::write(&path, xml).map_err(|e| e.to_string())?;

    Ok(())
}
