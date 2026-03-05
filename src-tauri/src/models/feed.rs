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
    pub healthy: bool,
    pub feed_type: FeedType,
    #[sea_orm(default_value = false)]
    pub deleted: bool,
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
