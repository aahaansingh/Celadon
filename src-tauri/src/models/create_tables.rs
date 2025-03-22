use sea_orm::{
    error::*, ConnectionTrait, DatabaseConnection, DbConn, EntityTrait, ExecResult, Schema,
};

async fn create_table<E>(db: &DbConn, entity: E) -> Result<(), DbErr>
where
    E: EntityTrait,
{
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let stmt = builder.build(&schema.create_table_from_entity(entity));

    db.execute(stmt).await?;
    Ok(())
}

pub async fn create_tables(db: &DbConn) -> Result<(), DbErr> {
    create_table(db, super::folder::Entity).await?;
    create_table(db, super::feed::Entity).await?;
    create_table(db, super::article::Entity).await?;
    create_table(db, super::tag::Entity).await?;
    create_table(db, super::tag_article::Entity).await?;
    Ok(())
}
