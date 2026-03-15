use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum FeedType {
    #[sea_orm(string_value = "News")]
    News,
    #[sea_orm(string_value = "Article")]
    Article,
    #[sea_orm(string_value = "Essay")]
    Essay,
    #[sea_orm(string_value = "Update")]
    Update,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "Feed")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub url: String,
    pub name: String,
    pub category: String,
    pub added: DateTime<Utc>,
        pub last_fetched: DateTime<Utc>,
        /// 0 = healthy, 1 = rate limited, 2–599 = HTTP error code
    pub status: i32,
    pub feed_type: FeedType,
    #[sea_orm(default_value = false)]
    pub deleted: bool,
    /// Sent as If-None-Match on next request
    pub etag: Option<String>,
    /// Sent as If-Modified-Since on next request (HTTP-date string)
    pub last_modified: Option<String>,
    /// When set and in the future, skip polling this feed
    pub next_poll_after: Option<DateTime<Utc>>,
    /// Number of consecutive 4xx/5xx errors: 0 = healthy, 1–2 = backoff (yellow), 3 = dead (red, stop polling)
    pub consecutive_http_errors: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::article::Entity")]
    Article,
}

impl Related<super::article::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Article.def()
    }
}

impl Related<super::superfeed::Entity> for Entity {
    fn to() -> RelationDef {
        super::feed_superfeed::Relation::Superfeed.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::feed_superfeed::Relation::Feed.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
