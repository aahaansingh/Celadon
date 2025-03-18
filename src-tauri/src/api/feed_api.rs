use crate::models::*;
use chrono::{DateTime, Utc};
use feed::ActiveModel;
use sea_orm::{
    entity::*, error::*, query::*, sea_query, tests_cfg::*, Database, DbConn, DeleteResult,
};

pub enum FeedStrFields {
    Url,
    Name,
    Category,
}

pub enum FeedDtFields {
    Added,
    LastFetched,
}

pub async fn get_feed(db: &DbConn, id: i32) -> Result<feed::Model, DbErr> {
    let retrieved_feed = Feed::find_by_id(id).one(db).await?;
    match retrieved_feed {
        None => Err(DbErr::RecordNotFound("No such feed exists".to_owned())),
        Some(feed_model) => Ok(feed_model),
    }
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
    folder: i32,
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
pub async fn update_feed_str(
    db: &DbConn,
    id: i32,
    val_type: FeedStrFields,
    new_val: String,
) -> Result<(), DbErr> {
    let feed_model = get_feed(db, id).await?;
    let mut feed_active: feed::ActiveModel = feed_model.into();
    match val_type {
        FeedStrFields::Url => feed_active.url = Set(new_val),
        FeedStrFields::Name => feed_active.name = Set(new_val),
        FeedStrFields::Category => feed_active.category = Set(new_val),
    }
    let _updated_feed_model = feed_active.update(db).await?;
    Ok(())
}

pub async fn update_feed_dt(
    db: &DbConn,
    id: i32,
    val_type: FeedDtFields,
    new_val: DateTime<Utc>,
) -> Result<(), DbErr> {
    let feed_model = get_feed(db, id).await?;
    let mut feed_active: feed::ActiveModel = feed_model.into();
    match val_type {
        FeedDtFields::Added => feed_active.added = Set(new_val),
        FeedDtFields::LastFetched => feed_active.last_fetched = Set(new_val),
    }
    let _updated_feed_model = feed_active.update(db).await?;
    Ok(())
}

pub async fn update_feed_folder(db: &DbConn, id: i32, new_val: i32) -> Result<(), DbErr> {
    let feed_model = get_feed(db, id).await?;
    let mut feed_active: feed::ActiveModel = feed_model.into();
    feed_active.folder = Set(new_val);
    let _updated_feed_model = feed_active.update(db).await?;
    Ok(())
}

pub async fn update_feed_health(db: &DbConn, id: i32, new_val: bool) -> Result<(), DbErr> {
    let feed_model = get_feed(db, id).await?;
    let mut feed_active: feed::ActiveModel = feed_model.into();
    feed_active.healthy = Set(new_val);
    let _updated_feed_model = feed_active.update(db).await?;
    Ok(())
}

pub async fn delete_feed(db: &DbConn, id: i32) -> Result<(), DbErr> {
    let res: DeleteResult = Feed::delete_by_id(id).exec(db).await?;
    if res.rows_affected != 1 {
        Err(DbErr::RecordNotFound("No such feed exists".to_owned()))
    } else {
        Ok(())
    }
}

// In this case, "none" indicates the retrieval of all articles; passing a number means
// adding a limit expression of that quantity
pub async fn get_articles(
    db: &DbConn,
    id: i32,
    num: Option<u64>,
) -> Result<Vec<article::Model>, DbErr> {
    let selected_feed = get_feed(db, id).await?;
    match num {
        None => {
            let related_articles = selected_feed
                .find_related(Article)
                .order_by(article::Column::Published, Order::Desc)
                .all(db)
                .await?;
            Ok(related_articles)
        }
        Some(lim) => {
            let related_articles = selected_feed
                .find_related(Article)
                .limit(lim)
                .order_by(article::Column::Published, Order::Desc)
                .all(db)
                .await?;
            Ok(related_articles)
        }
    }
}

pub async fn feed_max_id(db: &DbConn) -> Result<i32, DbErr> {
    let max_vec = Feed::find()
        .select_only()
        .column_as(feed::Column::Id.max(), "max_id")
        .into_tuple::<Option<i32>>()
        .one(db)
        .await?;
    match max_vec.unwrap() {
        None => Ok(0),
        Some(max) => Ok(max),
    }
}
