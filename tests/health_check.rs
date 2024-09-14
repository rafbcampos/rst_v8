#[tokio::test]
async fn health_check_works() {
    // Arrange
    spawn_app().await.expect("Failed to spawn our app");
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get("http://127.0.0.1:8000/health/full")
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    // response 200 with Ok as body
    assert_eq!(
        Some("Ok"),
        response.text_with_charset("utf-8").await.ok().as_deref()
    );
}

async fn spawn_app() -> Result<(), Box<dyn std::error::Error>> {
    let server = vm::run("127.0.0.1:8000").expect("Failed to bind address");
    tokio::spawn(server);
    Ok(())
}
