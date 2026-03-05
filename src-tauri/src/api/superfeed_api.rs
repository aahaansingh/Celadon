use crate::models::*;
use sea_orm::{
    entity::*, error::*, query::*, sea_query, tests_cfg::*, Database, DbBackend, DbConn,
    DeleteResult,
};

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
    match num {
        None => {
            let related_feeds = selected_superfeed
                .find_related(Feed)
                .order_by(feed::Column::Added, Order::Desc)
                .all(db)
                .await?;
            Ok(related_feeds)
        }
        Some(lim) => {
            let related_feeds = selected_superfeed
                .find_related(Feed)
                .limit(lim)
                .order_by(feed::Column::Added, Order::Desc)
                .all(db)
                .await?;
            Ok(related_feeds)
        }
    }
}
