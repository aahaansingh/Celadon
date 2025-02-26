use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "Tag")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub tag_id: i32,
    #[sea_orm(primary_key)]
    pub article_id: i32
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Tag,
    Article,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Article => Entity::belongs_to(super::article::Entity)
                .from(Column::ArticleId)
                .to(super::article::Column::Id)
                .into(),
            Self::Tag => Entity::belongs_to(super::tag::Entity)
                .from(Column::TagId)
                .to(super::tag::Column::Id)
                .into()
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}