use axum::{http::StatusCode, routing::post, Json, Router};
use clap::Parser;
use cp_guard::{dump_to_cp_dir, notify_proxy, BatchDumpRes, ParseResult};
use log::{debug, error};
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
struct Cli {
    userdir: String,
}

struct Bench {
    cli: Cli,
    tx: mpsc::Sender<BatchDumpRes>,
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);
    let bench = Arc::new(Bnech {
        cli: Cli::parse(),
        tx,
    });
    error!("cli: {:?}", &bench.cli);

    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new().route(
        "/",
        post({
            let bench = Arc::clone(&bench);
            move |body| handle_parse_result(body, bench)
        }),
    );

    // use impossible acmX port.
    error!("listening ...");
    let listener = tokio::net::TcpListener::bind("localhost:10042")
        .await
        .unwrap();
    error!("starting notify_proxy ...");
    tokio::spawn(async move || notify_proxy(rx).await);
    error!("serving ...");
    axum::serve(listener, app).await.unwrap();
}

#[allow(clippy::unused_async)]
async fn handle_parse_result(
    Json(payload): Json<ParseResult>,
    bench: Arc<Bench>,
) -> (StatusCode, &'static str) {
    debug!("userdir: {}, recv parse result: {payload:?}", cli.userdir);
    if let Err(e) = dump_to_cp_dir(&payload, &bench.cli.userdir, &bench.tx) {
        error!("dump_to_cp_dir error: {e:?}");
    }
    (StatusCode::CREATED, "Gotta")
}
