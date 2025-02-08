use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "Feed")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: i32,
    name: String
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Feed,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Feed => Entity::has_many(super::feed::Entity).into(),
        }
    }
}

impl Related<super::feed::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Feed.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}