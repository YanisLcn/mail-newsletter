use std::net::TcpListener;

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = mail_newsletter::run(listener).expect("Failed to bun address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{port}")
}

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn suscribe_returns_a_200_for_valid_form_data() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let body = "name=du%20bourg&email=arnaud_du_bourg%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=du%bourg", "missing email"),
        ("email=arnaud_du_bourg%40gmail.com", "missing name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let reponse = client
            .post(&format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-encoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            reponse.status().as_u16(),
            "The API did not fail with '400 Bad Request' when the payloads was {error_message}."
        );
    }
}