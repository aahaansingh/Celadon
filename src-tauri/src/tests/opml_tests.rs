#[cfg(test)]
mod opml_tests {
    use crate::api::{feed_api, opml_api, superfeed_api};
    use crate::models::create_tables;
    use crate::tests::utils::TestDB;

    #[tokio::test]
    async fn test_opml_import_export() {
        let test_db = TestDB::new("opml_test.db").await;
        let db = &test_db.db;
        create_tables::create_tables(db).await.unwrap();

        // Path to sample opml
        let mut sample_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        sample_path.push("src/tests/test_items/sample_opml.opml");
        let sample_path_str = sample_path.to_str().unwrap();

        // Test Import
        opml_api::import_opml_internal(db, sample_path_str.to_string())
            .await
            .expect("Import failed");

        // Verify superfeeds were created (Stationery, Culture, Sports)
        let superfeeds = superfeed_api::get_all_superfeeds(db).await.unwrap();
        // Stationeries, Culture, Sports have been added.
        // Note: Default superfeed ID 1 might exist if create_tables handles it,
        // but here we are checking the import results.
        assert!(
            superfeeds.len() >= 3,
            "Expected at least 3 superfeeds, found {}",
            superfeeds.len()
        );

        // Verify feeds were created
        let all_feeds = feed_api::get_all_feeds(db).await.unwrap();
        assert!(all_feeds.len() > 0, "Expected feeds to be imported");

        // Test Export
        let mut export_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        export_path.push("src/tests/test_items/exported.opml");
        let export_path_str = export_path.to_str().unwrap();

        opml_api::export_opml_internal(db, export_path_str.to_string())
            .await
            .expect("Export failed");

        // Cleanup
        let _ = std::fs::remove_file(export_path_str);
    }
}
