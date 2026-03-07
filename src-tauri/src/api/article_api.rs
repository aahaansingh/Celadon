use crate::models::article::Entity as Article;
use crate::models::article::ReadFilter;
use crate::models::{article, tag};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{InsertResult, QueryFilter, QueryOrder, QuerySelect, Set};

fn apply_read_filter<E>(query: Select<E>, filter: ReadFilter) -> Select<E>
where
    E: EntityTrait,
{
    match filter {
        ReadFilter::Unread => query.filter(article::Column::Read.eq(false)),
        ReadFilter::Read => query.filter(article::Column::Read.eq(true)),
        ReadFilter::All => query,
    }
}

pub async fn get_article(db: &DbConn, id: i32) -> Result<article::Model, DbErr> {
    let retrieved_article = Article::find_by_id(id)
        .filter(article::Column::Deleted.eq(false))
        .one(db)
        .await?;
    match retrieved_article {
        None => Err(DbErr::RecordNotFound("No such article exists".to_owned())),
        Some(article_model) => Ok(article_model),
    }
}

pub async fn get_article_by_url(db: &DbConn, url: String) -> Result<Option<article::Model>, DbErr> {
    let retrieved_articles = Article::find()
        .filter(article::Column::Url.eq(url))
        .filter(article::Column::Deleted.eq(false))
        .order_by_desc(article::Column::Published)
        .all(db)
        .await?;
    if retrieved_articles.len() > 1 {
        return Err(DbErr::Custom(
            "Multiple articles in database with same URL".to_owned(),
        ));
    }
    if retrieved_articles.len() == 0 {
        Ok(None)
    } else {
        Ok(Some(retrieved_articles[0].clone()))
    }
}

pub async fn create_article(
    db: &DbConn,
    id: i32,
    url: String,
    name: String,
    published: DateTime<Utc>,
    expiry_at: DateTime<Utc>,
    read: bool,
    description: String,
    feed: i32,
) -> Result<InsertResult<article::ActiveModel>, DbErr> {
    let insert = article::ActiveModel {
        id: Set(id),
        url: Set(url),
        name: Set(name),
        published: Set(published),
        expiry_at: Set(expiry_at),
        read: Set(read),
        description: Set(description),
        feed: Set(feed),
        ..Default::default()
    };

    Article::insert(insert).exec(db).await
}

pub async fn read_article(db: &DbConn, id: i32) -> Result<(), DbErr> {
    let article_model = get_article(db, id).await?;
    let mut article_active: article::ActiveModel = article_model.into();
    article_active.read = Set(true);
    let _updated_article_model = article_active.update(db).await?;
    Ok(())
}

pub async fn unread_article(db: &DbConn, id: i32) -> Result<(), DbErr> {
    let article_model = get_article(db, id).await?;
    let mut article_active: article::ActiveModel = article_model.into();
    article_active.read = Set(false);
    let _updated_article_model = article_active.update(db).await?;
    Ok(())
}

pub async fn read_all(db: &DbConn, feed_id: i32) -> Result<(), DbErr> {
    let _update_result = Article::update_many()
        .col_expr(article::Column::Read, Expr::value(true))
        .filter(article::Column::Feed.eq(feed_id))
        .filter(article::Column::Deleted.eq(false))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn delete_article(db: &DbConn, id: i32) -> Result<(), DbErr> {
    let article_model = get_article(db, id).await?;
    let mut article_active: article::ActiveModel = article_model.into();
    article_active.deleted = Set(true);
    article_active.update(db).await?;
    Ok(())
}

pub async fn undelete_article(db: &DbConn, id: i32) -> Result<(), DbErr> {
    let retrieved = Article::find_by_id(id).one(db).await?;
    if let Some(article_model) = retrieved {
        let mut article_active: article::ActiveModel = article_model.into();
        article_active.deleted = Set(false);
        article_active.update(db).await?;
    }
    Ok(())
}

pub async fn hard_delete_article(db: &DbConn, id: i32) -> Result<(), DbErr> {
    Article::delete_by_id(id).exec(db).await?;
    Ok(())
}

pub async fn cleanup_deleted_articles(db: &DbConn) -> Result<(), DbErr> {
    Article::delete_many()
        .filter(article::Column::Deleted.eq(true))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn clean_expired_articles(db: &DbConn) -> Result<(), DbErr> {
    let _update_result = Article::update_many()
        .col_expr(article::Column::Read, Expr::value(true))
        .filter(article::Column::ExpiryAt.lt(Utc::now()))
        .filter(article::Column::Read.eq(false))
        .filter(article::Column::Deleted.eq(false))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn article_max_id(db: &DbConn) -> Result<i32, DbErr> {
    let max_vec = Article::find()
        .select_only()
        .column_as(article::Column::Id.max(), "max_id")
        .into_tuple::<Option<i32>>()
        .one(db)
        .await?;
    match max_vec.unwrap() {
        None => Ok(0),
        Some(max) => Ok(max),
    }
}

pub async fn get_all_articles_sorted_relative(
    db: &DbConn,
    filter: ReadFilter,
    num: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<article::Model>, DbErr> {
    let mut query = Article::find().filter(article::Column::Deleted.eq(false));

    query = apply_read_filter(query, filter);

    query = query.order_by_asc(Expr::cust("CAST(strftime('%s', 'now') - strftime('%s', published) AS FLOAT) / CAST(strftime('%s', expiry_at) - strftime('%s', published) AS FLOAT)"));

    if let Some(lim) = num {
        query = query.limit(lim);
    }
    if let Some(off) = offset {
        query = query.offset(off);
    }

    query.all(db).await
}

pub async fn search_articles(
    db: &DbConn,
    search_query: String,
    filter: ReadFilter,
    num: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<article::Model>, DbErr> {
    let mut query = Article::find()
        .filter(article::Column::Deleted.eq(false))
        .filter(
            article::Column::Name
                .contains(&search_query)
                .or(article::Column::Description.contains(&search_query)),
        );

    query = apply_read_filter(query, filter);

    query = query.order_by_asc(Expr::cust("CAST(strftime('%s', 'now') - strftime('%s', published) AS FLOAT) / CAST(strftime('%s', expiry_at) - strftime('%s', published) AS FLOAT)"));

    if let Some(lim) = num {
        query = query.limit(lim);
    }
    if let Some(off) = offset {
        query = query.offset(off);
    }

    query.all(db).await
}

pub async fn get_tags(db: &DbConn, id: i32) -> Result<Vec<tag::Model>, DbErr> {
    let related_article_tags = Article::find()
        .filter(article::Column::Id.eq(id))
        .find_with_related(tag::Entity)
        .all(db)
        .await?;

    match related_article_tags.len() {
        1 => {
            let tags = related_article_tags[0]
                .1
                .clone()
                .into_iter()
                .filter(|t| !t.deleted)
                .collect();
            Ok(tags)
        }
        _ => Err(DbErr::RecordNotFound("No such article exists".to_owned())),
    }
}
