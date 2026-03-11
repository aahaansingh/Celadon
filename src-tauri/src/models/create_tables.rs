use sea_orm::{ConnectionTrait, DbConn, DbErr, EntityTrait, Schema};

async fn create_table<E>(db: &DbConn, entity: E) -> Result<(), DbErr>
where
    E: EntityTrait,
{
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let mut table_create_statement = schema.create_table_from_entity(entity);
    table_create_statement.if_not_exists();
    let stmt = builder.build(&table_create_statement);

    db.execute(stmt).await?;
    Ok(())
}

pub async fn create_tables(db: &DbConn) -> Result<(), DbErr> {
    create_table(db, super::superfeed::Entity).await?;
    create_table(db, super::feed::Entity).await?;
    create_table(db, super::feed_superfeed::Entity).await?;
    create_table(db, super::article::Entity).await?;
    create_table(db, super::tag::Entity).await?;
    create_table(db, super::tag_article::Entity).await?;

    // Create Article indices for performance
    db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_articles_sorting ON Article(deleted, read, published DESC);"
            .to_owned(),
    ))
    .await?;

    db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_articles_feed_filter ON Article(feed, deleted, read, published DESC);"
            .to_owned(),
    ))
    .await?;

    // Enforce at most one non-deleted feed per URL
    db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_feed_url_not_deleted ON Feed(url) WHERE deleted = 0;"
            .to_owned(),
    ))
    .await?;

    Ok(())
}
