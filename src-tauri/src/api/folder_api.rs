use crate::models::*;
use chrono::{DateTime, Utc};
use folder::ActiveModel;
use sea_orm::{
    entity::*, error::*, query::*, sea_query, tests_cfg::*, Database, DbBackend, DbConn,
    DeleteResult,
};

pub async fn get_folder(db: &DbConn, id: i32) -> Result<folder::Model, DbErr> {
    let retrieved_folder = Folder::find_by_id(id).one(db).await?;
    match retrieved_folder {
        None => Err(DbErr::RecordNotFound("No such folder exists".to_owned())),
        Some(folder_model) => Ok(folder_model),
    }
}

pub async fn create_folder(db: &DbConn, id: i32, name: String) -> InsertResult<ActiveModel> {
    let insert = folder::ActiveModel {
        id: Set(id),
        name: Set(name),
        ..Default::default()
    };

    let insert_suc = Folder::insert(insert)
        .exec(db)
        .await
        .expect("couldn't insert folder");
    insert_suc
}

pub async fn rename_folder(db: &DbConn, id: i32, name: String) -> Result<(), DbErr> {
    let folder_model = get_folder(db, id).await?;
    let mut folder_active: folder::ActiveModel = folder_model.into();
    folder_active.name = Set(name);
    let _updated_folder_model = folder_active.update(db).await?;
    Ok(())
}

pub async fn delete_folder(db: &DbConn, id: i32) -> Result<Option<()>, DbErr> {
    if id == 1 {
        // You can't delete the main folder
        return Ok(None);
    }
    let res: DeleteResult = Folder::delete_by_id(id).exec(db).await?;
    if res.rows_affected != 1 {
        Err(DbErr::RecordNotFound("No such folder exists".to_owned()))
    } else {
        Ok(Some(()))
    }
}

pub async fn folder_max_id(db: &DbConn) -> Result<i32, DbErr> {
    let max_vec = Folder::find()
        .select_only()
        .column_as(folder::Column::Id.max(), "max_id")
        .into_tuple::<Option<i32>>()
        .one(db)
        .await?;
    match max_vec.unwrap() {
        None => Ok(0),
        Some(max) => Ok(max),
    }
}

pub async fn get_feeds(db: &DbConn, id: i32, num: Option<u64>) -> Result<Vec<feed::Model>, DbErr> {
    let selected_folder = get_folder(db, id).await?;
    match num {
        None => {
            let related_feeds = selected_folder
                .find_related(Feed)
                .order_by(feed::Column::Added, Order::Desc)
                .all(db)
                .await?;
            Ok(related_feeds)
        }
        Some(lim) => {
            let related_feeds = selected_folder
                .find_related(Feed)
                .limit(lim)
                .order_by(feed::Column::Added, Order::Desc)
                .all(db)
                .await?;
            Ok(related_feeds)
        }
    }
}
