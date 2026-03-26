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
    create_table(db, super::settings::Entity).await?;

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

    // Enforce at most one non-deleted feed per URL (should be redundant)
    db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_feed_url_not_deleted ON Feed(url) WHERE deleted = 0;"
            .to_owned(),
    ))
    .await?;

    create_fts_tables(db).await?;

    Ok(())
}

/// Create FTS5 virtual tables and triggers for Article, Feed, Superfeed, Tag. Idempotent (IF NOT EXISTS).
async fn create_fts_tables(db: &DbConn) -> Result<(), DbErr> {
    use sea_orm::Statement;

    let backend = db.get_database_backend();

    // Article FTS: name, description
    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE VIRTUAL TABLE IF NOT EXISTS article_fts USING fts5(name, description, content='Article', content_rowid='id');"
            .to_owned(),
    ))
    .await?;

    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE TRIGGER IF NOT EXISTS article_fts_insert AFTER INSERT ON Article BEGIN
            INSERT INTO article_fts(rowid, name, description) VALUES (new.id, new.name, new.description);
        END;"
            .to_owned(),
    ))
    .await?;
    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE TRIGGER IF NOT EXISTS article_fts_delete AFTER DELETE ON Article BEGIN
            INSERT INTO article_fts(article_fts, rowid, name, description) VALUES ('delete', old.id, old.name, old.description);
        END;"
            .to_owned(),
    ))
    .await?;
    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE TRIGGER IF NOT EXISTS article_fts_update AFTER UPDATE ON Article BEGIN
            INSERT INTO article_fts(article_fts, rowid, name, description) VALUES ('delete', old.id, old.name, old.description);
            INSERT INTO article_fts(rowid, name, description) VALUES (new.id, new.name, new.description);
        END;"
            .to_owned(),
    ))
    .await?;

    // Feed FTS: name
    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE VIRTUAL TABLE IF NOT EXISTS feed_fts USING fts5(name, content='Feed', content_rowid='id');"
            .to_owned(),
    ))
    .await?;
    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE TRIGGER IF NOT EXISTS feed_fts_insert AFTER INSERT ON Feed BEGIN
            INSERT INTO feed_fts(rowid, name) VALUES (new.id, new.name);
        END;"
            .to_owned(),
    ))
    .await?;
    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE TRIGGER IF NOT EXISTS feed_fts_delete AFTER DELETE ON Feed BEGIN
            INSERT INTO feed_fts(feed_fts, rowid, name) VALUES ('delete', old.id, old.name);
        END;"
            .to_owned(),
    ))
    .await?;
    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE TRIGGER IF NOT EXISTS feed_fts_update AFTER UPDATE ON Feed BEGIN
            INSERT INTO feed_fts(feed_fts, rowid, name) VALUES ('delete', old.id, old.name);
            INSERT INTO feed_fts(rowid, name) VALUES (new.id, new.name);
        END;"
            .to_owned(),
    ))
    .await?;

    // Superfeed FTS: name
    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE VIRTUAL TABLE IF NOT EXISTS superfeed_fts USING fts5(name, content='Superfeed', content_rowid='id');"
            .to_owned(),
    ))
    .await?;
    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE TRIGGER IF NOT EXISTS superfeed_fts_insert AFTER INSERT ON Superfeed BEGIN
            INSERT INTO superfeed_fts(rowid, name) VALUES (new.id, new.name);
        END;"
            .to_owned(),
    ))
    .await?;
    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE TRIGGER IF NOT EXISTS superfeed_fts_delete AFTER DELETE ON Superfeed BEGIN
            INSERT INTO superfeed_fts(superfeed_fts, rowid, name) VALUES ('delete', old.id, old.name);
        END;"
            .to_owned(),
    ))
    .await?;
    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE TRIGGER IF NOT EXISTS superfeed_fts_update AFTER UPDATE ON Superfeed BEGIN
            INSERT INTO superfeed_fts(superfeed_fts, rowid, name) VALUES ('delete', old.id, old.name);
            INSERT INTO superfeed_fts(rowid, name) VALUES (new.id, new.name);
        END;"
            .to_owned(),
    ))
    .await?;

    // Tag FTS: name
    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE VIRTUAL TABLE IF NOT EXISTS tag_fts USING fts5(name, content='Tag', content_rowid='id');"
            .to_owned(),
    ))
    .await?;
    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE TRIGGER IF NOT EXISTS tag_fts_insert AFTER INSERT ON Tag BEGIN
            INSERT INTO tag_fts(rowid, name) VALUES (new.id, new.name);
        END;"
            .to_owned(),
    ))
    .await?;
    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE TRIGGER IF NOT EXISTS tag_fts_delete AFTER DELETE ON Tag BEGIN
            INSERT INTO tag_fts(tag_fts, rowid, name) VALUES ('delete', old.id, old.name);
        END;"
            .to_owned(),
    ))
    .await?;
    db.execute(Statement::from_string(
        backend.clone(),
        "CREATE TRIGGER IF NOT EXISTS tag_fts_update AFTER UPDATE ON Tag BEGIN
            INSERT INTO tag_fts(tag_fts, rowid, name) VALUES ('delete', old.id, old.name);
            INSERT INTO tag_fts(rowid, name) VALUES (new.id, new.name);
        END;"
            .to_owned(),
    ))
    .await?;

    // Initial sync: rebuild FTS from content tables (idempotent; safe to run when tables already populated)
    db.execute(Statement::from_string(
        backend.clone(),
        "INSERT INTO article_fts(article_fts) VALUES('rebuild');".to_owned(),
    ))
    .await
    .ok();
    db.execute(Statement::from_string(
        backend.clone(),
        "INSERT INTO feed_fts(feed_fts) VALUES('rebuild');".to_owned(),
    ))
    .await
    .ok();
    db.execute(Statement::from_string(
        backend.clone(),
        "INSERT INTO superfeed_fts(superfeed_fts) VALUES('rebuild');".to_owned(),
    ))
    .await
    .ok();
    db.execute(Statement::from_string(
        backend.clone(),
        "INSERT INTO tag_fts(tag_fts) VALUES('rebuild');".to_owned(),
    ))
    .await
    .ok();

    Ok(())
}

/// Rebuild all FTS5 indexes from their content tables. Call after ensuring default data (e.g. "All" superfeed) exists so superfeed_fts is populated.
pub async fn rebuild_fts_indexes(db: &DbConn) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    db.execute(sea_orm::Statement::from_string(
        backend.clone(),
        "INSERT INTO article_fts(article_fts) VALUES('rebuild');".to_owned(),
    ))
    .await
    .ok();
    db.execute(sea_orm::Statement::from_string(
        backend.clone(),
        "INSERT INTO feed_fts(feed_fts) VALUES('rebuild');".to_owned(),
    ))
    .await
    .ok();
    db.execute(sea_orm::Statement::from_string(
        backend.clone(),
        "INSERT INTO superfeed_fts(superfeed_fts) VALUES('rebuild');".to_owned(),
    ))
    .await
    .ok();
    db.execute(sea_orm::Statement::from_string(
        backend.clone(),
        "INSERT INTO tag_fts(tag_fts) VALUES('rebuild');".to_owned(),
    ))
    .await
    .ok();
    Ok(())
}
