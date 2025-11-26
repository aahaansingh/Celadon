use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::ForeignKeyAction;
use serde::Serialize;

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
    pub folder: i32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Article,
    Folder,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Article => Entity::has_many(super::article::Entity).into(),
            Self::Folder => Entity::belongs_to(super::folder::Entity)
                .from(Column::Folder)
                .to(super::folder::Column::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .into(),
        }
    }
}

impl Related<super::article::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Article.def()
    }
}

impl Related<super::folder::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Folder.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
