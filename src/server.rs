// Copyright 2022 Oxide Computer Company
//! Example use of Dropshot with a websocket endpoint.

use std::net::Ipv4Addr;
use std::net::SocketAddrV4;

use dropshot::channel;
use dropshot::ApiDescription;
use dropshot::ConfigDropshot;
use dropshot::ConfigLogging;
use dropshot::ConfigLoggingLevel;
use dropshot::ConfigTls;
use dropshot::HttpServerStarter;
use dropshot::Query;
use dropshot::RequestContext;
use dropshot::WebsocketConnection;
use futures::SinkExt;
use schemars::JsonSchema;
use serde::Deserialize;
use tokio_tungstenite::tungstenite::protocol::Role;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() -> Result<(), String> {
    // We must specify a configuration with a bind address.  We'll use 127.0.0.1
    // since it's available and won't expose this server outside the host.  We
    // request port 0, which allows the operating system to pick any available
    // port.
    let port: u16 = std::env::var("LISTEN_PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap();
    let config_dropshot = ConfigDropshot {
        bind_address: std::net::SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(127, 0, 0, 1),
            port,
        )),
        request_body_max_bytes: 1024,
        tls: Some(ConfigTls::AsFile {
            cert_file: "certs/server.crt".to_string().into(),
            key_file: "certs/server.key".to_string().into(),
        }),
    };

    // For simplicity, we'll configure an "info"-level logger that writes to
    // stderr assuming that it's a terminal.
    let config_logging = ConfigLogging::StderrTerminal {
        level: ConfigLoggingLevel::Info,
    };
    let log = config_logging
        .to_logger("example-basic")
        .map_err(|error| format!("failed to create logger: {}", error))?;

    // Build a description of the API.
    let mut api = ApiDescription::new();
    api.register(example_api_websocket_counter).unwrap();

    // Set up the server.
    let server = HttpServerStarter::new(&config_dropshot, api, (), &log)
        .map_err(|error| format!("failed to create server: {}", error))?
        .start();

    // Wait for the server to stop.  Note that there's not any code to shut down
    // this server, so we should never get past this point.
    server.await
}

// HTTP API interface

#[derive(Deserialize, JsonSchema)]
struct QueryParams {
    start: Option<u8>,
}

/// An eternally-increasing sequence of bytes, wrapping on overflow, starting
/// from the value given for the query parameter "start."
#[channel {
    protocol = WEBSOCKETS,
    path = "/counter",
}]
async fn example_api_websocket_counter(
    rqctx: RequestContext<()>,
    qp: Query<QueryParams>,
    upgraded: WebsocketConnection,
) -> dropshot::WebsocketChannelResult {
    let protocol = rqctx.request.version();
    slog::info!(rqctx.log, "websocket connection established"; "protocol" => ?protocol);
    let mut ws = tokio_tungstenite::WebSocketStream::from_raw_socket(
        upgraded.into_inner(),
        Role::Server,
        None,
    )
    .await;
    let mut count = qp.into_inner().start.unwrap_or(0);
    while ws.send(Message::Binary(vec![count])).await.is_ok() {
        count = count.wrapping_add(1);
    }
    Ok(())
}
