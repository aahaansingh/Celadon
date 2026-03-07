use crate::api::{feed_api, superfeed_api};
use crate::models::feed::FeedType;
use chrono::Utc;
use opml::{Body, Head, Outline, OPML};
use sea_orm::DbConn;

pub async fn import_opml_internal(db: &DbConn, path: String) -> Result<(), String> {
    let xml = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    let document = OPML::from_str(&xml).map_err(|e| e.to_string())?;

    let now = Utc::now();

    for outline in document.body.outlines {
        if outline.xml_url.is_some() {
            let feed_id = feed_api::feed_max_id(db).await.unwrap_or(0) + 1;
            let url = outline.xml_url.clone().unwrap_or_default();
            let name = outline.text.clone();
            let _ = feed_api::create_feed(
                db,
                feed_id,
                url,
                name,
                "Import".to_string(),
                now,
                now,
                true,
                FeedType::News,
            )
            .await;
            let _ = feed_api::add_feed_to_superfeed(db, feed_id, 1).await;
        } else {
            let superfeed_name = outline
                .title
                .clone()
                .unwrap_or_else(|| outline.text.clone());
            let sf_id = superfeed_api::superfeed_max_id(db).await.unwrap_or(0) + 1;
            let _ = superfeed_api::create_superfeed(db, sf_id, superfeed_name).await;

            for sub_outline in outline.outlines {
                if sub_outline.xml_url.is_some() {
                    let feed_id = feed_api::feed_max_id(db).await.unwrap_or(0) + 1;
                    let url = sub_outline.xml_url.clone().unwrap_or_default();
                    let name = sub_outline.text.clone();
                    let _ = feed_api::create_feed(
                        db,
                        feed_id,
                        url,
                        name,
                        "Import".to_string(),
                        now,
                        now,
                        true,
                        FeedType::News,
                    )
                    .await;
                    let _ = feed_api::add_feed_to_superfeed(db, feed_id, sf_id).await;
                }
            }
        }
    }

    Ok(())
}

pub async fn export_opml_internal(db: &DbConn, path: String) -> Result<(), String> {
    let mut root_outlines = Vec::new();
    let superfeeds = superfeed_api::get_all_superfeeds(db)
        .await
        .map_err(|e| e.to_string())?;

    for sf in superfeeds {
        let feeds = superfeed_api::get_feeds(db, sf.id, None)
            .await
            .map_err(|e| e.to_string())?;

        let mut child_outlines = Vec::new();
        for f in feeds {
            child_outlines.push(Outline {
                text: f.name.clone(),
                title: Some(f.name.clone()),
                xml_url: Some(f.url.clone()),
                ..Default::default()
            });
        }

        root_outlines.push(Outline {
            text: sf.name.clone(),
            title: Some(sf.name.clone()),
            outlines: child_outlines,
            ..Default::default()
        });
    }

    let opml = OPML {
        version: "2.0".to_string(),
        head: Some(Head {
            title: Some("Celadon OPML Export".to_string()),
            ..Default::default()
        }),
        body: Body {
            outlines: root_outlines,
        },
    };

    let xml = opml.to_string().map_err(|e| e.to_string())?;
    std::fs::write(&path, xml).map_err(|e| e.to_string())?;

    Ok(())
}
