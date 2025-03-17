use article::ActiveModel;
use prelude::Expr;
use sea_orm::{entity::*, error::*, query::*, sea_query, tests_cfg::*, Database, DbConn, DeleteResult};
use chrono::{DateTime, Utc};
use crate::models::*;
use super::feed_api::get_feed;

pub async fn get_article(db: &DbConn, id: i32) -> Result<article::Model, DbErr> {
    let retrieved_article = Article::find_by_id(id).one(db).await?;
    match retrieved_article {
        None => Err(DbErr::RecordNotFound("No such article exists".to_owned())),
        Some(article_model) => Ok(article_model)
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
        feed: i32
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

pub async fn read_all(db: &DbConn, folder_id: i32) -> Result<(), DbErr> {
    let _update_result = Article::update_many()
        .col_expr(article::Column::Read, Expr::value(true))
        .filter(article::Column::Feed.eq(folder_id))
        .exec(db)
        .await?;
    Ok(())
}