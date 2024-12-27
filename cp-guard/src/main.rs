use axum::{http::StatusCode, routing::post, Json, Router};
use clap::Parser;
use cp_guard::{dump_to_cp_dir, ParseResult};
use log::{debug, error};
use std::sync::Arc;

#[derive(Parser, Debug)]
struct Cli {
    userdir: String,
}

#[tokio::main]
async fn main() {
    let cli = Arc::new(Cli::parse());
    error!("cli: {:?}", &cli);

    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new().route(
        "/",
        post({
            let cli = Arc::clone(&cli);
            move |body| handle_parse_result(body, cli)
        }),
    );

    // use impossible acmX port.
    error!("listening ...");
    let listener = tokio::net::TcpListener::bind("localhost:10042")
        .await
        .unwrap();
    error!("serving ...");
    axum::serve(listener, app).await.unwrap();
}

#[allow(clippy::unused_async)]
async fn handle_parse_result(
    Json(payload): Json<ParseResult>,
    cli: Arc<Cli>,
) -> (StatusCode, &'static str) {
    debug!("userdir: {}, recv parse result: {payload:?}", cli.userdir);
    if let Err(e) = dump_to_cp_dir(&payload, &cli.userdir) {
        error!("dump_to_cp_dir error: {e:?}");
    }
    (StatusCode::CREATED, "Gotta")
}
