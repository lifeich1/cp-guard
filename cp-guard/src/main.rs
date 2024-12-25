use axum::{http::StatusCode, routing::post, Json, Router};
use cp_guard::ParseResult;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new().route("/", post(handle_parse_result));

    // use impossible acmX port.
    println!("listening ...");
    let listener = tokio::net::TcpListener::bind("localhost:10042")
        .await
        .unwrap();
    println!("serving ...");
    axum::serve(listener, app).await.unwrap();
}

async fn handle_parse_result(Json(payload): Json<ParseResult>) -> (StatusCode, &'static str) {
    println!("recv parse result: {payload:?}");
    (StatusCode::CREATED, "Gotta")
}
