use crate::models::article::Entity as Article;
use crate::models::article::ReadFilter;
use crate::models::feed::Entity as Feed;
use crate::models::superfeed::Entity as Superfeed;
use crate::models::{article, feed, superfeed};
use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait, InsertResult, JoinType, Order, QueryFilter, QueryOrder, QuerySelect, Set};

pub async fn get_superfeed(db: &DbConn, id: i32) -> Result<superfeed::Model, DbErr> {
    let retrieved_superfeed = Superfeed::find_by_id(id)
        .filter(superfeed::Column::Deleted.eq(false))
        .one(db)
        .await?;
    match retrieved_superfeed {
        None => Err(DbErr::RecordNotFound("No such superfeed exists".to_owned())),
        Some(superfeed_model) => Ok(superfeed_model),
    }
}

pub async fn get_all_superfeeds(db: &DbConn) -> Result<Vec<superfeed::Model>, DbErr> {
    Superfeed::find()
        .filter(superfeed::Column::Deleted.eq(false))
        .all(db)
        .await
}

pub async fn create_superfeed(
    db: &DbConn,
    id: i32,
    name: String,
) -> Result<InsertResult<superfeed::ActiveModel>, DbErr> {
    if name.contains('\\') {
        return Err(DbErr::Custom("Name cannot contain backslashes".to_owned()));
    }
    let insert = superfeed::ActiveModel {
        id: Set(id),
        name: Set(name),
        ..Default::default()
    };

    Superfeed::insert(insert).exec(db).await
}

pub async fn rename_superfeed(db: &DbConn, id: i32, name: String) -> Result<(), DbErr> {
    if name.contains('\\') {
        return Err(DbErr::Custom("Name cannot contain backslashes".to_owned()));
    }
    let superfeed_model = get_superfeed(db, id).await?;
    let mut superfeed_active: superfeed::ActiveModel = superfeed_model.into();
    superfeed_active.name = Set(name);
    let _updated_superfeed_model = superfeed_active.update(db).await?;
    Ok(())
}

pub async fn delete_superfeed(db: &DbConn, id: i32) -> Result<(), DbErr> {
    if id == 1 {
        // You can't delete the main superfeed
        return Err(DbErr::Custom("Cannot delete default superfeed.".to_owned()));
    }
    let superfeed_model = get_superfeed(db, id).await?;
    let mut superfeed_active: superfeed::ActiveModel = superfeed_model.into();
    superfeed_active.deleted = Set(true);
    superfeed_active.update(db).await?;
    Ok(())
}

pub async fn undelete_superfeed(db: &DbConn, id: i32) -> Result<(), DbErr> {
    let retrieved = Superfeed::find_by_id(id).one(db).await?;
    if let Some(superfeed_model) = retrieved {
        let mut superfeed_active: superfeed::ActiveModel = superfeed_model.into();
        superfeed_active.deleted = Set(false);
        superfeed_active.update(db).await?;
    }
    Ok(())
}

pub async fn hard_delete_superfeed(db: &DbConn, id: i32) -> Result<(), DbErr> {
    Superfeed::delete_by_id(id).exec(db).await?;
    Ok(())
}

pub async fn cleanup_deleted_superfeeds(db: &DbConn) -> Result<(), DbErr> {
    Superfeed::delete_many()
        .filter(superfeed::Column::Deleted.eq(true))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn superfeed_max_id(db: &DbConn) -> Result<i32, DbErr> {
    let max_vec = Superfeed::find()
        .select_only()
        .column_as(superfeed::Column::Id.max(), "max_id")
        .into_tuple::<Option<i32>>()
        .one(db)
        .await?;
    match max_vec.unwrap() {
        None => Ok(0),
        Some(max) => Ok(max),
    }
}

pub async fn get_feeds(db: &DbConn, id: i32, num: Option<u64>) -> Result<Vec<feed::Model>, DbErr> {
    let selected_superfeed = get_superfeed(db, id).await?;
    let mut query = selected_superfeed
        .find_related(Feed)
        .order_by(feed::Column::Added, Order::Desc);

    if let Some(lim) = num {
        query = query.limit(lim);
    }

    query.all(db).await
}

pub async fn get_articles(
    db: &DbConn,
    id: i32,
    filter: ReadFilter,
    num: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<article::Model>, DbErr> {
    let _selected_superfeed = get_superfeed(db, id).await?;

    let mut query = Article::find()
        .join(JoinType::InnerJoin, article::Relation::Feed.def())
        .join(
            JoinType::InnerJoin,
            crate::models::feed_superfeed::Relation::Feed.def().rev(),
        )
        .filter(crate::models::feed_superfeed::Column::SuperfeedId.eq(id))
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

pub async fn search_superfeeds(db: &DbConn, query: String) -> Result<Vec<superfeed::Model>, DbErr> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }
    let terms: Vec<String> = trimmed
        .split_whitespace()
        .map(|s| s.replace('"', "\"\""))
        .filter(|s| !s.is_empty())
        .collect();
    if terms.is_empty() {
        return Ok(vec![]);
    }
    let fts_query = terms
        .iter()
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(" OR ");
    let backend = db.get_database_backend();
    let stmt = sea_orm::Statement::from_sql_and_values(
        backend,
        "SELECT rowid FROM superfeed_fts WHERE superfeed_fts MATCH ?",
        [sea_orm::Value::String(Some(Box::new(fts_query)))],
    );
    let rows = db.query_all(stmt).await?;
    let ids: Vec<i32> = rows
        .into_iter()
        .filter_map(|r| r.try_get_by_index::<i32>(0).ok())
        .collect();
    if ids.is_empty() {
        return Ok(vec![]);
    }
    Superfeed::find()
        .filter(superfeed::Column::Id.is_in(ids))
        .filter(superfeed::Column::Deleted.eq(false))
        .all(db)
        .await
}
