use sea_orm::entity::prelude::*;
use sea_orm::sea_query::ForeignKeyAction;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "TagArticle")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub tag_id: i32,
    #[sea_orm(primary_key)]
    pub article_id: i32,
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
                .on_delete(ForeignKeyAction::Cascade)
                .into(),
            Self::Tag => Entity::belongs_to(super::tag::Entity)
                .from(Column::TagId)
                .to(super::tag::Column::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
