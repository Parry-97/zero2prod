use std::net::TcpListener;

use zero2prod::startup::run;

//NOTE: `tokio::test` is the testing equivalent of `tokio::main`.
//You can inspect the generated code using `cargo expand --test health_check (<- name of the file)`
#[tokio::test]
async fn health_check_works() {
    let client = reqwest::Client::new();
    let address = spawn_app();

    let response = client
        .get(&format!("{address}/health_check"))
        .send()
        .await
        .expect("Failed to execute request. ");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let client = reqwest::Client::new();
    let address = spawn_app();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{address}/subscriptions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request. ");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let client = reqwest::Client::new();
    let address = spawn_app();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{address}/subscriptions"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request. ");

        assert_eq!(400,response.status().as_u16(),
        "The API did not fail but returned 400 Bad Request when the payload was {error_message}");
    }
}

//NOTE: This function is the only piece in our tests that depends on the application code.
//Everything else is decoupled from the underlying implementation details
fn spawn_app() -> String {
    //WARN: when we call await on `run` it starts listening ,on the address we specified,
    //indefinetely. It never returns and our test logic never gets executed.

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind address");
    // zero2prod::run().await
    //NOTE: We need to use `tokio::spawn` to run it as a background task
    tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
