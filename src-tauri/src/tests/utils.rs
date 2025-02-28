use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub struct TestDB {
    url: String,
    db_name: String,
    pub db: DatabaseConnection,
}

impl TestDB {
    pub async fn new(db_name: &str) -> Self {
        // let url = std::env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' not set");
        let url = "sqlite::memory:".to_string();
        let mut options: ConnectOptions = (&url).into();
        options.sqlx_logging(false);
        let db = Database::connect(options).await.unwrap();
        Self {
            url,
            db_name: db_name.to_string(),
            db
        }
    }
    pub async fn delete(&self) {
        
    }
}