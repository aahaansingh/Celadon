use feed::ActiveModel;
use sea_orm::{entity::*, error::*, query::*, sea_query, tests_cfg::*, Database, DbConn};
use chrono::{DateTime, Utc};
use crate::models::*;

pub enum FeedStrFields {
    Url,
    Name,
    Category,
}

pub enum FeedDtFields {
    Added,
    LastFetched,
}

pub async fn get_feed(db: &DbConn, id: i32) -> Result<Option<feed::Model>, DbErr> {
    let retrieved_feed = Feed::find_by_id(id).one(db).await;
    retrieved_feed
}

pub async fn create_feed(
    db: &DbConn, 
    id: i32,
    url: String,
    name: String,
    category: String,
    added: DateTime<Utc>,
    last_fetched: DateTime<Utc>,
    healthy: bool,
    folder: i32
    ) -> InsertResult<ActiveModel> {
        let insert = feed::ActiveModel {
            id: Set(id),
            url: Set(url.to_owned()),
            name: Set(name.to_owned()),
            category: Set(category.to_owned()),
            added: Set(added),
            last_fetched: Set(last_fetched),
            healthy: Set(healthy),
            folder: Set(folder),
            ..Default::default()
        };

        let insert_suc = Feed::insert(insert)
        .exec(db)
        .await
        .expect("couldn't insert feed");
    insert_suc
}

// Setters
pub async fn update_feed_str(db: &DbConn, id: i32, val_type: FeedStrFields, new_val: String) -> Result<(), DbErr> {
    let retrieved_feed = Feed::find_by_id(id).one(db).await?;
    match retrieved_feed {
        None => return Err(DbErr::RecordNotFound("No such feed exists".to_owned())),
        Some(feed_model) => {
            let mut feed_active: feed::ActiveModel = feed_model.into();
            match val_type {
                FeedStrFields::Url => feed_active.url = Set(new_val),
                FeedStrFields::Name => feed_active.name = Set(new_val),
                FeedStrFields::Category => feed_active.category = Set(new_val),
            }
            let _updated_feed_model = feed_active.update(db).await?;
            Ok(())
        }
    }
}

pub async fn update_feed_dt(db: &DbConn, id: i32, val_type: FeedDtFields, new_val: DateTime<Utc>) -> Result<(), DbErr> {
    let retrieved_feed = Feed::find_by_id(id).one(db).await?;
    match retrieved_feed {
        None => return Err(DbErr::RecordNotFound("No such feed exists".to_owned())),
        Some(feed_model) => {
            let mut feed_active: feed::ActiveModel = feed_model.into();
            match val_type {
                FeedDtFields::Added => feed_active.added = Set(new_val),
                FeedDtFields::LastFetched => feed_active.last_fetched = Set(new_val)
            }
            let _updated_feed_model = feed_active.update(db).await?;
            Ok(())
        }
    }
}

pub async fn update_feed_folder(db: &DbConn, id: i32, new_val: i32) -> Result<(), DbErr> {
    let retrieved_feed = Feed::find_by_id(id).one(db).await?;
    match retrieved_feed {
        None => return Err(DbErr::RecordNotFound("No such feed exists".to_owned())),
        Some(feed_model) => {
            let mut feed_active: feed::ActiveModel = feed_model.into();
            feed_active.folder = Set(new_val);
            let _updated_feed_model = feed_active.update(db).await?;
            Ok(())
        }
    }
}

pub async fn update_feed_health(db: &DbConn, id: i32, new_val: bool) -> Result<(), DbErr> {
    let retrieved_feed = Feed::find_by_id(id).one(db).await?;
    match retrieved_feed {
        None => return Err(DbErr::RecordNotFound("No such feed exists".to_owned())),
        Some(feed_model) => {
            let mut feed_active: feed::ActiveModel = feed_model.into();
            feed_active.healthy = Set(new_val);
            let _updated_feed_model = feed_active.update(db).await?;
            Ok(())
        }
    }
}