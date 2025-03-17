#[cfg(test)]
mod basic_tests {
    use chrono::Utc;
    use sea_orm::{
        entity::prelude::*, entity::*, tests_cfg::*, DatabaseBackend, MockDatabase, Transaction,
    };

    use crate::models::*;

    #[async_std::test]
    async fn test_get_feed() -> Result<(), DbErr> {
        /// Simple test against MockDB to ensure proper insertion/return
        let add = Utc::now();
        let fetch = Utc::now();
        // DB Inserts
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![folder::Model {
                id: 1,
                name: "root".to_owned(),
            }]])
            .append_query_results([vec![
                feed::Model {
                    id: 1,
                    url: "http://www.osearch.org/feed".to_owned(),
                    name: "OSearch".to_owned(),
                    category: "Science".to_owned(),
                    added: add,
                    last_fetched: fetch,
                    healthy: true,
                    folder: 1,
                },
                feed::Model {
                    id: 2,
                    url: "https://feeds.kottke.org/main".to_owned(),
                    name: "Kottke".to_owned(),
                    category: "Blog".to_owned(),
                    added: add,
                    last_fetched: fetch,
                    healthy: true,
                    folder: 1,
                },
            ]])
            .append_query_results([[(
                folder::Model {
                    id: 1,
                    name: "root".to_owned(),
                },
                feed::Model {
                    id: 1,
                    url: "http://www.osearch.org/feed".to_owned(),
                    name: "OSearch".to_owned(),
                    category: "Science".to_owned(),
                    added: add,
                    last_fetched: fetch,
                    healthy: true,
                    folder: 1,
                },
            )]])
            .into_connection();

        assert_eq!(
            Folder::find().one(&db).await?,
            Some(folder::Model {
                id: 1,
                name: "root".to_owned()
            })
        );

        assert_eq!(
            Feed::find().all(&db).await?,
            [
                feed::Model {
                    id: 1,
                    url: "http://www.osearch.org/feed".to_owned(),
                    name: "OSearch".to_owned(),
                    category: "Science".to_owned(),
                    added: add,
                    last_fetched: fetch,
                    healthy: true,
                    folder: 1
                },
                feed::Model {
                    id: 2,
                    url: "https://feeds.kottke.org/main".to_owned(),
                    name: "Kottke".to_owned(),
                    category: "Blog".to_owned(),
                    added: add,
                    last_fetched: fetch,
                    healthy: true,
                    folder: 1
                }
            ]
        );

        assert_eq!(
            Folder::find()
                .find_also_related(feed::Entity)
                .all(&db)
                .await?,
            [(
                folder::Model {
                    id: 1,
                    name: "root".to_owned()
                },
                Some(feed::Model {
                    id: 1,
                    url: "http://www.osearch.org/feed".to_owned(),
                    name: "OSearch".to_owned(),
                    category: "Science".to_owned(),
                    added: add,
                    last_fetched: fetch,
                    healthy: true,
                    folder: 1
                })
            )]
        );
        Ok(())
    }
}
