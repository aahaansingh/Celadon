use crate::models::*;
use chrono::{DateTime, Utc};
use sea_orm::{
    entity::*, error::*, query::*, sea_query, tests_cfg::*, Database, DbConn, DeleteResult,
};
use tag::*;
use tag_article::*;

pub async fn get_tag(db: &DbConn, id: i32) -> Result<tag::Model, DbErr> {
    let retrieved_tag = Tag::find_by_id(id).one(db).await?;
    match retrieved_tag {
        None => Err(DbErr::RecordNotFound("No such tag exists".to_owned())),
        Some(tag_model) => Ok(tag_model),
    }
}

pub async fn get_all_tags(db: &DbConn) -> Result<Vec<tag::Model>, DbErr> {
    Tag::find().all(db).await
}

pub async fn create_tag(db: &DbConn, id: i32, name: String) -> InsertResult<tag::ActiveModel> {
    let insert = tag::ActiveModel {
        id: Set(id),
        name: Set(name),
        ..Default::default()
    };

    let insert_suc = Tag::insert(insert)
        .exec(db)
        .await
        .expect("couldn't insert tag");
    insert_suc
}

pub async fn tag_article(
    db: &DbConn,
    tag_id: i32,
    article_id: i32,
) -> InsertResult<tag_article::ActiveModel> {
    let insert = tag_article::ActiveModel {
        tag_id: Set(tag_id),
        article_id: Set(article_id),
    };

    let insert_suc = TagArticle::insert(insert)
        .exec(db)
        .await
        .expect("couldn't tag this article");
    insert_suc
}

pub async fn delete_tag_article(db: &DbConn, tag_id: i32, article_id: i32) -> Result<(), DbErr> {
    let res: DeleteResult = TagArticle::delete_by_id((tag_id, article_id))
        .exec(db)
        .await?;
    if res.rows_affected != 1 {
        Err(DbErr::RecordNotFound(
            "This tag-article pair does not exist".to_owned(),
        ))
    } else {
        Ok(())
    }
}

pub async fn rename_tag(db: &DbConn, id: i32, name: String) -> Result<(), DbErr> {
    let tag_model = get_tag(db, id).await?;
    let mut tag_active: tag::ActiveModel = tag_model.into();
    tag_active.name = Set(name);
    let _updated_tag_model = tag_active.update(db).await?;
    Ok(())
}

pub async fn delete_tag(db: &DbConn, id: i32) -> Result<(), DbErr> {
    let res: DeleteResult = Tag::delete_by_id(id).exec(db).await?;
    if res.rows_affected != 1 {
        Err(DbErr::RecordNotFound("No such tag exists".to_owned()))
    } else {
        Ok(())
    }
}

pub async fn tag_max_id(db: &DbConn) -> Result<i32, DbErr> {
    let max_vec = Tag::find()
        .select_only()
        .column_as(tag::Column::Id.max(), "max_id")
        .into_tuple::<Option<i32>>()
        .one(db)
        .await?;
    match max_vec.unwrap() {
        None => Ok(0),
        Some(max) => Ok(max),
    }
}

pub async fn get_articles(db: &DbConn, id: i32) -> Result<Vec<article::Model>, DbErr> {
    let related_tag_articles = Tag::find()
        .filter(tag::Column::Id.eq(id))
        .find_with_related(Article)
        .all(db)
        .await?;
    match related_tag_articles.len() {
        1 => {
            Ok(related_tag_articles[0].1.clone()) // Not sure if I should clone...
        }
        _ => Err(DbErr::RecordNotFound("No such tag exists".to_owned())),
    }
}
