//! tests/ai/ollama_provider.rs

use codeatlas::ai::ollama_provider::OllamaProvider;
use mockito::mock;

#[tokio::test]
async fn test_ollama_provider_infer_success() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let _m = mock("POST", "/api/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"response": "This is a test response"}"#)
        .create_async(&mut server).await;

    let provider = OllamaProvider::new("test-model", &url);
    let response = provider.infer("test prompt").await.unwrap();

    assert_eq!(response, "This is a test response");
}

#[tokio::test]
async fn test_ollama_provider_infer_api_error() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let _m = mock("POST", "/api/generate")
        .with_status(500)
        .create_async(&mut server).await;

    let provider = OllamaProvider::new("test-model", &url);
    let result = provider.infer("test prompt").await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Ollama API request failed with status: 500 Internal Server Error");
}

#[tokio::test]
async fn test_ollama_provider_check_status_success() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let _m = mock("GET", "/api/tags")
        .with_status(200)
        .create_async(&mut server).await;

    let provider = OllamaProvider::new("test-model", &url);
    let result = provider.check_status().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ollama_provider_check_status_error() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();

    let _m = mock("GET", "/api/tags")
        .with_status(500)
        .create_async(&mut server).await;

    let provider = OllamaProvider::new("test-model", &url);
    let result = provider.check_status().await;

    assert!(result.is_err());
}