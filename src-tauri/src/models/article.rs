use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::ForeignKeyAction;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "Article")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub url: String,
    pub name: String,
    pub published: DateTime<Utc>,
    pub read: bool,
    pub description: String,
    pub feed: i32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Feed,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Feed => Entity::belongs_to(super::feed::Entity)
                .from(Column::Feed)
                .to(super::feed::Column::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .into(),
        }
    }
}

impl Related<super::feed::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Feed.def()
    }
}

impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        super::tag_article::Relation::Tag.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::tag_article::Relation::Article.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
