use crate::models::article::Entity as Article;
use crate::models::article::ReadFilter;
use crate::models::tag_article;
use crate::models::{article, tag};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{Expr as SeaExpr, Query};
use sea_orm::{ConnectionTrait, InsertResult, QueryFilter, QueryOrder, QuerySelect, Set};

/// Maximum number of non-deleted articles to retain; oldest untagged are pruned when over.
pub const ARTICLE_CAP: u32 = 100_000;
/// Articles older than this many days are eligible for retention delete if untagged.
pub const RETENTION_DAYS: i64 = 365;

// Sort by "urgency" ratio (now-published)/(expiry-published). When expiry_at <= published
// the ratio would be NULL/division-by-zero; use 1.0 so those rows sort at end.
const RELATIVE_SORT: &str = "CASE WHEN (strftime('%s', expiry_at) - strftime('%s', published)) > 0 THEN (CAST(strftime('%s', 'now') - strftime('%s', published) AS FLOAT) / CAST(strftime('%s', expiry_at) - strftime('%s', published) AS FLOAT)) ELSE 1.0 END";

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

/// Count of non-deleted articles.
pub async fn article_count(db: &DbConn) -> Result<u64, DbErr> {
    let count = Article::find()
        .filter(article::Column::Deleted.eq(false))
        .count(db)
        .await?;
    Ok(count)
}

/// Subquery: article_id from TagArticle join Tag where Tag.deleted = 0 (ids of articles that have at least one non-deleted tag).
fn tagged_article_ids_subquery() -> sea_orm::sea_query::SelectStatement {
    Query::select()
        .column((tag_article::Entity, tag_article::Column::ArticleId))
        .from(tag_article::Entity)
        .inner_join(
            tag::Entity,
            SeaExpr::col((tag_article::Entity, tag_article::Column::TagId))
                .equals((tag::Entity, tag::Column::Id)),
        )
        .and_where(SeaExpr::col((tag::Entity, tag::Column::Deleted)).eq(false))
        .to_owned()
}

/// Delete non-deleted articles older than RETENTION_DAYS that have no (non-deleted) tags.
pub async fn delete_articles_older_than_retention(db: &DbConn) -> Result<u64, DbErr> {
    let cutoff = Utc::now() - chrono::TimeDelta::days(RETENTION_DAYS);
    let result = Article::delete_many()
        .filter(article::Column::Deleted.eq(false))
        .filter(article::Column::Published.lt(cutoff))
        .filter(article::Column::Id.not_in_subquery(tagged_article_ids_subquery()))
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}

/// If article count exceeds max_count, delete oldest-by-published untagged articles until count <= max_count.
pub async fn ensure_article_cap(db: &DbConn, max_count: u32) -> Result<u64, DbErr> {
    let count = article_count(db).await?;
    if count <= max_count as u64 {
        return Ok(0);
    }
    let to_remove = (count - max_count as u64) as u64;
    let ids_to_remove: Vec<i32> = Article::find()
        .select_only()
        .column(article::Column::Id)
        .filter(article::Column::Deleted.eq(false))
        .filter(article::Column::Id.not_in_subquery(tagged_article_ids_subquery()))
        .order_by_asc(article::Column::Published)
        .limit(to_remove)
        .into_tuple::<i32>()
        .all(db)
        .await?;
    if ids_to_remove.is_empty() {
        return Ok(0);
    }
    let result = Article::delete_many()
        .filter(article::Column::Id.is_in(ids_to_remove))
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}

/// Backfill expiry_at for articles where expiry_at <= published (e.g. old data) so the relative sort works.
pub async fn backfill_expiry_at(db: &DbConn) -> Result<(), DbErr> {
    use sea_orm::Statement;
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "UPDATE Article SET expiry_at = datetime(published, '+1 day') WHERE deleted = 0 AND expiry_at <= published".to_owned(),
    ))
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

    query = query.order_by_asc(Expr::cust(RELATIVE_SORT));

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
    let trimmed = search_query.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }
    // FTS5: build OR of quoted terms so "foo bar" matches docs containing "foo" OR "bar".
    // Escape internal double quotes and drop terms that would be empty after escaping.
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
        "SELECT rowid FROM article_fts WHERE article_fts MATCH ?",
        [sea_orm::Value::String(Some(Box::new(fts_query)))],
    );
    let rows = db.query_all(stmt).await?;
    let mut ids: Vec<i32> = Vec::with_capacity(rows.len());
    for row in rows {
        let id: i32 = row.try_get_by_index(0)?;
        ids.push(id);
    }
    if ids.is_empty() {
        return Ok(vec![]);
    }
    // Cap ids to avoid huge IN clause; we'll sort and paginate in memory
    const MAX_FTS_IDS: usize = 10_000;
    if ids.len() > MAX_FTS_IDS {
        ids.truncate(MAX_FTS_IDS);
    }
    let mut query = Article::find()
        .filter(article::Column::Id.is_in(ids))
        .filter(article::Column::Deleted.eq(false));
    query = apply_read_filter(query, filter);
    let mut articles = query.all(db).await?;
    // Sort by relative urgency (same as RELATIVE_SORT)
    let now_secs = Utc::now().timestamp();
    articles.sort_by(|a, b| {
        let sort_val = |art: &article::Model| {
            let pub_secs = art.published.timestamp();
            let exp_secs = art.expiry_at.timestamp();
            let range = exp_secs - pub_secs;
            if range > 0 {
                (now_secs - pub_secs) as f64 / range as f64
            } else {
                1.0
            }
        };
        sort_val(a)
            .partial_cmp(&sort_val(b))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let offset = offset.unwrap_or(0) as usize;
    let num = num.unwrap_or(50) as usize;
    let skip = offset.min(articles.len());
    let take = num.min(articles.len().saturating_sub(skip));
    Ok(articles.into_iter().skip(skip).take(take).collect())
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
