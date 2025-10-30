//! Unit tests for MonitoringDatabase (UV-219)

use uveddi::monitoring::database::MonitoringDatabase;
use uveddi::config::monitoring::MonitoringConfig;

#[tokio::test]
async fn test_monitoring_database_new() {
    let config = MonitoringConfig::default();
    let db = MonitoringDatabase::new(&config.database_url).await;
    assert!(db.is_ok(), "MonitoringDatabase should initialize successfully");
}
