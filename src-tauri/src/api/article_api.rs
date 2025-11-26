use super::feed_api::get_feed;
use crate::models::*;
use article::ActiveModel;
use chrono::{DateTime, Utc};
use prelude::Expr;
use sea_orm::{
    entity::*, error::*, query::*, sea_query, tests_cfg::*, Database, DbConn, DeleteResult,
};

pub async fn get_article(db: &DbConn, id: i32) -> Result<article::Model, DbErr> {
    let retrieved_article = Article::find_by_id(id).one(db).await?;
    match retrieved_article {
        None => Err(DbErr::RecordNotFound("No such article exists".to_owned())),
        Some(article_model) => Ok(article_model),
    }
}

// Again, I should have keyed by URL...
// Not because feeds can be uniquely identified by URL but because dealing wih linkless articles
// is beyond the bounds of necessity
pub async fn get_article_by_url(db: &DbConn, url: String) -> Result<Option<article::Model>, DbErr> {
    let retrieved_articles = Article::find()
        .filter(article::Column::Url.eq(url))
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
    read: bool,
    description: String,
    feed: i32,
) -> InsertResult<ActiveModel> {
    let insert = article::ActiveModel {
        id: Set(id),
        url: Set(url),
        name: Set(name),
        published: Set(published),
        read: Set(read),
        description: Set(description),
        feed: Set(feed),
        ..Default::default()
    };

    let insert_suc = Article::insert(insert)
        .exec(db)
        .await
        .expect("couldn't insert article");
    insert_suc
}

pub async fn read_article(db: &DbConn, id: i32) -> Result<(), DbErr> {
    let article_model = get_article(db, id).await?;
    let mut article_active: article::ActiveModel = article_model.into();
    article_active.read = Set(true);
    let _updated_article_model = article_active.update(db).await?;
    Ok(())
}

pub async fn read_all(db: &DbConn, feed_id: i32) -> Result<(), DbErr> {
    let _update_result = Article::update_many()
        .col_expr(article::Column::Read, Expr::value(true))
        .filter(article::Column::Feed.eq(feed_id))
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

pub async fn get_tags(db: &DbConn, id: i32) -> Result<Vec<tag::Model>, DbErr> {
    let related_article_tags = Article::find()
        .filter(article::Column::Id.eq(id))
        .find_with_related(Tag)
        .all(db)
        .await?;

    match related_article_tags.len() {
        1 => {
            Ok(related_article_tags[0].1.clone()) // Again, cloning may not be right here
        }
        _ => Err(DbErr::RecordNotFound("No such article exists".to_owned())),
    }
}
