use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "Tag")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Related<super::article::Entity> for Entity {
    fn to() -> RelationDef {
        super::tag_article::Relation::Article.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::tag_article::Relation::Tag.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
