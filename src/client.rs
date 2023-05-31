use reqwest::StatusCode;

#[tokio::main]
async fn main() {
    env_logger::init();
    let client = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let port: u16 = std::env::var("LISTEN_PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap();
    let req = client
        .get(format!("https://0.0.0.0:{port}/counter"))
        .header(reqwest::header::CONNECTION, "Upgrade")
        .header(reqwest::header::UPGRADE, "websocket")
        .header(reqwest::header::SEC_WEBSOCKET_VERSION, "13")
        .header(
            reqwest::header::SEC_WEBSOCKET_KEY,
            base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                rand::random::<[u8; 16]>(),
            ),
        )
        .build()
        .unwrap();

    // Send the request
    let resp = client.execute(req).await.unwrap();
    let status = resp.status();
    let body = resp.text().await.unwrap();
    if status != StatusCode::SWITCHING_PROTOCOLS {
        eprintln!("Bad status {}", status);
        eprintln!("Body:\n{}", body);
    }
}
