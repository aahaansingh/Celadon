use crate::models::article::Entity as Article;
use crate::models::article::ReadFilter;
use crate::models::feed::Entity as Feed;
use crate::models::feed_superfeed::Entity as FeedSuperfeed;
use crate::models::{article, feed, feed_superfeed};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::prelude::Expr;
use sea_orm::{InsertResult, QueryFilter, QueryOrder, QuerySelect, Set};

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
    let retrieved_feed = Feed::find_by_id(id)
        .filter(feed::Column::Deleted.eq(false))
        .one(db)
        .await?;
    match retrieved_feed {
        None => Err(DbErr::RecordNotFound("No such feed exists".to_owned())),
        Some(feed_model) => Ok(feed_model),
    }
}

pub async fn get_all_feeds(db: &DbConn) -> Result<Vec<feed::Model>, DbErr> {
    Feed::find()
        .filter(feed::Column::Deleted.eq(false))
        .all(db)
        .await
}

pub async fn get_feed_by_url(db: &DbConn, url: String) -> Result<Option<feed::Model>, DbErr> {
    let retrieved_feeds = Feed::find()
        .filter(feed::Column::Url.eq(url))
        .filter(feed::Column::Deleted.eq(false))
        .all(db)
        .await?;
    if retrieved_feeds.len() > 1 {
        return Err(DbErr::Custom(
            "Multiple feeds in database with same URL".to_owned(),
        ));
    }
    if retrieved_feeds.len() == 0 {
        Ok(None)
    } else {
        Ok(Some(retrieved_feeds[0].clone()))
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
    feed_type: feed::FeedType,
) -> Result<InsertResult<feed::ActiveModel>, DbErr> {
    if name.contains('\\') {
        return Err(DbErr::Custom("Name cannot contain backslashes".to_owned()));
    }
    let insert = feed::ActiveModel {
        id: Set(id),
        url: Set(url.to_owned()),
        name: Set(name.to_owned()),
        category: Set(category.to_owned()),
        added: Set(added),
        last_fetched: Set(last_fetched),
        healthy: Set(healthy),
        feed_type: Set(feed_type),
        ..Default::default()
    };

    Feed::insert(insert).exec(db).await
}

pub async fn update_feed_str(
    db: &DbConn,
    id: i32,
    val_type: FeedStrFields,
    new_val: String,
) -> Result<(), DbErr> {
    if matches!(val_type, FeedStrFields::Name) && new_val.contains('\\') {
        return Err(DbErr::Custom("Name cannot contain backslashes".to_owned()));
    }
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

pub async fn update_feed_type(
    db: &DbConn,
    id: i32,
    feed_type: feed::FeedType,
) -> Result<(), DbErr> {
    let feed_model = get_feed(db, id).await?;
    let mut feed_active: feed::ActiveModel = feed_model.into();
    feed_active.feed_type = Set(feed_type);
    let _updated_feed_model = feed_active.update(db).await?;
    Ok(())
}

pub async fn add_feed_to_superfeed(
    db: &DbConn,
    feed_id: i32,
    superfeed_id: i32,
) -> Result<(), DbErr> {
    let relation = feed_superfeed::ActiveModel {
        feed_id: Set(feed_id),
        superfeed_id: Set(superfeed_id),
    };
    FeedSuperfeed::insert(relation).exec(db).await?;
    Ok(())
}

pub async fn get_superfeed_ids_for_feed(db: &DbConn, feed_id: i32) -> Result<Vec<i32>, DbErr> {
    let rows = feed_superfeed::Entity::find()
        .filter(feed_superfeed::Column::FeedId.eq(feed_id))
        .all(db)
        .await?;
    Ok(rows.into_iter().map(|r| r.superfeed_id).collect())
}

pub async fn remove_feed_from_superfeed(
    db: &DbConn,
    feed_id: i32,
    superfeed_id: i32,
) -> Result<(), DbErr> {
    FeedSuperfeed::delete_by_id((feed_id, superfeed_id))
        .exec(db)
        .await?;
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
    let feed_model = get_feed(db, id).await?;
    let mut feed_active: feed::ActiveModel = feed_model.into();
    feed_active.deleted = Set(true);
    feed_active.update(db).await?;
    // Soft-delete all articles in this feed so they don't show as "Unknown Feed" and can be restored on undo
    Article::update_many()
        .col_expr(article::Column::Deleted, Expr::value(true))
        .filter(article::Column::Feed.eq(id))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn undelete_feed(db: &DbConn, id: i32) -> Result<(), DbErr> {
    let retrieved = Feed::find_by_id(id).one(db).await?;
    if let Some(feed_model) = retrieved {
        let mut feed_active: feed::ActiveModel = feed_model.into();
        feed_active.deleted = Set(false);
        feed_active.update(db).await?;
    }
    // Restore all articles that belonged to this feed and were soft-deleted with it
    Article::update_many()
        .col_expr(article::Column::Deleted, Expr::value(false))
        .filter(article::Column::Feed.eq(id))
        .filter(article::Column::Deleted.eq(true))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn hard_delete_feed(db: &DbConn, id: i32) -> Result<(), DbErr> {
    Feed::delete_by_id(id).exec(db).await?;
    Ok(())
}

pub async fn cleanup_deleted_feeds(db: &DbConn) -> Result<(), DbErr> {
    Feed::delete_many()
        .filter(feed::Column::Deleted.eq(true))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn get_articles(
    db: &DbConn,
    id: i32,
    filter: ReadFilter,
    num: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<article::Model>, DbErr> {
    let selected_feed = get_feed(db, id).await?;
    let mut query = selected_feed
        .find_related(Article)
        .filter(article::Column::Deleted.eq(false));

    match filter {
        ReadFilter::Unread => {
            query = query.filter(article::Column::Read.eq(false));
        }
        ReadFilter::Read => {
            query = query.filter(article::Column::Read.eq(true));
        }
        ReadFilter::All => {}
    }

    query = query.order_by_asc(Expr::cust("CASE WHEN (strftime('%s', expiry_at) - strftime('%s', published)) > 0 THEN (CAST(strftime('%s', 'now') - strftime('%s', published) AS FLOAT) / CAST(strftime('%s', expiry_at) - strftime('%s', published) AS FLOAT)) ELSE 1.0 END"));

    if let Some(lim) = num {
        query = query.limit(lim);
    }
    if let Some(off) = offset {
        query = query.offset(off);
    }

    query.all(db).await
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

pub async fn search_feeds(db: &DbConn, query: String) -> Result<Vec<feed::Model>, DbErr> {
    Feed::find()
        .filter(feed::Column::Name.contains(&query))
        .filter(feed::Column::Deleted.eq(false))
        .all(db)
        .await
}
