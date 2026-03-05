use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "FeedSuperfeed")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub feed_id: i32,
    #[sea_orm(primary_key)]
    pub superfeed_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::feed::Entity",
        from = "Column::FeedId",
        to = "super::feed::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Feed,
    #[sea_orm(
        belongs_to = "super::superfeed::Entity",
        from = "Column::SuperfeedId",
        to = "super::superfeed::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Superfeed,
}

impl ActiveModelBehavior for ActiveModel {}
