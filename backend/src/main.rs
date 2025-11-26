use std::time::Duration;

use axum::{
    Json, Router,
    http::{Method, StatusCode, header::CONTENT_TYPE},
    response::IntoResponse,
    routing::get,
};
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    info!("Starting server...");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE])
        .max_age(Duration::from_secs(60 * 60));

    let app = Router::new()
        .route("/hello", get(hello_handler))
        .layer(cors);

    let addr = format!("0.0.0.0:{}", 1111);
    info!("Binding to {}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();

    info!("Server running on {}", addr);

    axum::serve(listener, app).await.unwrap();

    println!("Server shutting down...");
}

#[derive(Deserialize)]
struct Token {
    token: String,
}

#[axum::debug_handler]
async fn hello_handler(Json(payload): Json<Token>) -> impl IntoResponse {
    (StatusCode::OK, payload.token).into_response()
}
