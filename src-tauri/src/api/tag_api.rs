use crate::models::article::Entity as Article;
use crate::models::article::ReadFilter;
use crate::models::tag::Entity as Tag;
use crate::models::tag_article::Entity as TagArticle;
use crate::models::{article, tag, tag_article};
use sea_orm::entity::prelude::*;
use sea_orm::{
    ConnectionTrait, DeleteResult, InsertResult, JoinType, QueryFilter, QueryOrder, QuerySelect,
    Set,
};

pub async fn get_tag(db: &DbConn, id: i32) -> Result<tag::Model, DbErr> {
    let retrieved_tag = Tag::find_by_id(id)
        .filter(tag::Column::Deleted.eq(false))
        .one(db)
        .await?;
    match retrieved_tag {
        None => Err(DbErr::RecordNotFound("No such tag exists".to_owned())),
        Some(tag_model) => Ok(tag_model),
    }
}

pub async fn get_all_tags(db: &DbConn) -> Result<Vec<tag::Model>, DbErr> {
    Tag::find()
        .filter(tag::Column::Deleted.eq(false))
        .all(db)
        .await
}

pub async fn create_tag(
    db: &DbConn,
    id: i32,
    name: String,
) -> Result<InsertResult<tag::ActiveModel>, DbErr> {
    if name.contains('\\') {
        return Err(DbErr::Custom("Name cannot contain backslashes".to_owned()));
    }
    let insert = tag::ActiveModel {
        id: Set(id),
        name: Set(name),
        ..Default::default()
    };

    Tag::insert(insert).exec(db).await
}

pub async fn tag_article(
    db: &DbConn,
    tag_id: i32,
    article_id: i32,
) -> Result<InsertResult<tag_article::ActiveModel>, DbErr> {
    let insert = tag_article::ActiveModel {
        tag_id: Set(tag_id),
        article_id: Set(article_id),
    };

    TagArticle::insert(insert).exec(db).await
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
    if name.contains('\\') {
        return Err(DbErr::Custom("Name cannot contain backslashes".to_owned()));
    }
    let tag_model = get_tag(db, id).await?;
    let mut tag_active: tag::ActiveModel = tag_model.into();
    tag_active.name = Set(name);
    let _updated_tag_model = tag_active.update(db).await?;
    Ok(())
}

pub async fn delete_tag(db: &DbConn, id: i32) -> Result<(), DbErr> {
    let tag_model = get_tag(db, id).await?;
    let mut tag_active: tag::ActiveModel = tag_model.into();
    tag_active.deleted = Set(true);
    tag_active.update(db).await?;
    Ok(())
}

pub async fn undelete_tag(db: &DbConn, id: i32) -> Result<(), DbErr> {
    let retrieved = Tag::find_by_id(id).one(db).await?;
    if let Some(tag_model) = retrieved {
        let mut tag_active: tag::ActiveModel = tag_model.into();
        tag_active.deleted = Set(false);
        tag_active.update(db).await?;
    }
    Ok(())
}

pub async fn hard_delete_tag(db: &DbConn, id: i32) -> Result<(), DbErr> {
    Tag::delete_by_id(id).exec(db).await?;
    Ok(())
}

pub async fn cleanup_deleted_tags(db: &DbConn) -> Result<(), DbErr> {
    Tag::delete_many()
        .filter(tag::Column::Deleted.eq(true))
        .exec(db)
        .await?;
    Ok(())
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

pub async fn get_articles(
    db: &DbConn,
    id: i32,
    filter: ReadFilter,
    num: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<article::Model>, DbErr> {
    let _selected_tag = get_tag(db, id).await?;

    let mut query = Article::find()
        .join(
            JoinType::InnerJoin,
            tag_article::Relation::Article.def().rev(),
        )
        .filter(tag_article::Column::TagId.eq(id))
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

pub async fn search_tags(db: &DbConn, query: String) -> Result<Vec<tag::Model>, DbErr> {
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
        "SELECT rowid FROM tag_fts WHERE tag_fts MATCH ?",
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
    Tag::find()
        .filter(tag::Column::Id.is_in(ids))
        .filter(tag::Column::Deleted.eq(false))
        .all(db)
        .await
}
