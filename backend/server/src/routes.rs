use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Token {
    token: String,
}

pub async fn hello_handler(Json(payload): Json<Token>) -> impl IntoResponse {
    (StatusCode::OK, payload.token).into_response()
}

pub async fn search_handler(Json(payload): Json<Token>) -> impl IntoResponse {
    (StatusCode::OK, payload.token).into_response()
}
