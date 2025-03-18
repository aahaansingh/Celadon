use crate::models::*;
use chrono::{DateTime, Utc};
use tag::ActiveModel;
use sea_orm::{
    entity::*, error::*, query::*, sea_query, tests_cfg::*, Database, DbConn, DeleteResult,
};

pub async fn get_tag(db: &DbConn, id: i32) -> Result<tag::Model, DbErr> {
    let retrieved_tag = Tag::find_by_id(id).one(db).await?;
    match retrieved_tag {
        None => Err(DbErr::RecordNotFound("No such tag exists".to_owned())),
        Some(tag_model) => Ok(tag_model),
    }
}

pub async fn create_tag(db: &DbConn, id: i32, name: String) -> InsertResult<ActiveModel> {
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
        .one(db)
        .await?;
    match max_vec {
        None => Ok(0),
        Some(max) => Ok(max.id),
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
        },
        _ => Err(DbErr::RecordNotFound("No such tag exists".to_owned()))
    }

}