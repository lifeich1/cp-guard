use axum::{http::StatusCode, routing::post, Json, Router};
use clap::Parser;
use cp_guard::ParseResult;
use std::sync::Arc;

#[derive(Parser, Debug)]
struct Cli {
    userdir: String,
}

#[tokio::main]
async fn main() {
    let cli = Arc::new(Cli::parse());
    println!("cli: {:?}", &cli);

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
    println!("listening ...");
    let listener = tokio::net::TcpListener::bind("localhost:10042")
        .await
        .unwrap();
    println!("serving ...");
    axum::serve(listener, app).await.unwrap();
}

#[allow(clippy::unused_async)]
async fn handle_parse_result(
    Json(payload): Json<ParseResult>,
    cli: Arc<Cli>,
) -> (StatusCode, &'static str) {
    println!("userdir: {}, recv parse result: {payload:?}", cli.userdir);
    (StatusCode::CREATED, "Gotta")
}
