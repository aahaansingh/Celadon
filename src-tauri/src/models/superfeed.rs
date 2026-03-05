use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "Superfeed")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub name: String,
    #[sea_orm(default_value = false)]
    pub deleted: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Related<super::feed::Entity> for Entity {
    fn to() -> RelationDef {
        super::feed_superfeed::Relation::Feed.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::feed_superfeed::Relation::Superfeed.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
