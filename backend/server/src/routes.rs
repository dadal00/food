use std::sync::Arc;

use axum::{Json, body::Bytes, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
#[cfg(feature = "verbose")]
use tracing::info;

use crate::{
    database::update_foods, error::AppError, state::State as AppState, utils::get_votes_from_body,
};

#[derive(Deserialize)]
pub struct Token {
    token: String,
}

pub async fn votes_handler(
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let votes = get_votes_from_body(&state, body)?;

    #[cfg(feature = "verbose")]
    info!("Length of votes: {}", votes.len());
    update_foods(&mut state.redis_connection.clone(), &votes).await?;

    Ok((StatusCode::OK, "Accepted").into_response())
}

pub async fn search_handler(Json(payload): Json<Token>) -> impl IntoResponse {
    (StatusCode::OK, payload.token).into_response()
}
