use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

// pub struct Article {
//     pub id: i32,
//     pub url: String,
//     pub name: String,
//     pub feed: i32, // Foreign
//     pub published: DateTime<Utc>,
//     pub read: bool,
//     pub description: String
// }

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "Article")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub url: String,
    pub name: String,
    pub published: DateTime<Utc>,
    pub read: bool,
    pub description: String
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Feed
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Feed => Entity::belongs_to(super::feed::Entity).into()
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

