//! tests/cli/init_local_ai_command.rs

use uveddi::cli::init_local_ai_command::{InitLocalAiCommand, LocalAiSetup};
use async_trait::async_trait;
use mockall::mock;

mock! {
    pub LocalAiSetup {}

    #[async_trait]
    impl LocalAiSetup for LocalAiSetup {
        async fn is_ollama_installed(&self) -> bool;
        async fn is_ollama_running(&self) -> bool;
        async fn download_model(&self, model: &str);
    }
}

#[tokio::test]
async fn test_init_local_ai_ollama_not_installed() {
    let mut mock_setup = MockLocalAiSetup::new();
    mock_setup.expect_is_ollama_installed().returning(|| false).once();

    let command = InitLocalAiCommand { model: "test-model".to_string() };
    command.execute(&mock_setup).await.unwrap();
}

#[tokio::test]
async fn test_init_local_ai_ollama_not_running() {
    let mut mock_setup = MockLocalAiSetup::new();
    mock_setup.expect_is_ollama_installed().returning(|| true).once();
    mock_setup.expect_is_ollama_running().returning(|| false).once();

    let command = InitLocalAiCommand { model: "test-model".to_string() };
    command.execute(&mock_setup).await.unwrap();
}

#[tokio::test]
async fn test_init_local_ai_success() {
    let mut mock_setup = MockLocalAiSetup::new();
    mock_setup.expect_is_ollama_installed().returning(|| true).once();
    mock_setup.expect_is_ollama_running().returning(|| true).once();
    mock_setup.expect_download_model().withf(|m| m == "test-model").once();

    let command = InitLocalAiCommand { model: "test-model".to_string() };
    command.execute(&mock_setup).await.unwrap();
}