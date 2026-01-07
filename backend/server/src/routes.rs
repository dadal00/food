use std::sync::Arc;

use axum::{Json, body::Bytes, extract::State, http::StatusCode, response::IntoResponse};
use bank::{get_votes_from_bytes, payloads::Votes};
use serde::Deserialize;
#[cfg(feature = "verbose")]
use tracing::info;

use crate::{
    database::{Vote, update_foods},
    error::AppError::{self, MalformedPayload},
    state::State as AppState,
};

#[derive(Deserialize)]
pub struct Token {
    token: String,
}

fn get_maps(state: Arc<AppState>, bytes: Bytes) -> Result<Votes, AppError> {
    let vote_maps = get_votes_from_bytes(bytes).map_err(|_| AppError::MalformedPayload)?;

    if vote_maps.old_bit_map.len() != vote_maps.new_bit_map.len()
        || vote_maps.old_bit_map.len() > state.remote_bank.food_id_to_name.len()
    {
        return Err(AppError::MalformedPayload);
    }

    Ok(vote_maps)
}

pub async fn votes_handler(
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let vote_maps = get_maps(state.clone(), body)?;

    let mut votes = Vec::new();

    for (byte_index, (&old_byte, &new_byte)) in vote_maps
        .old_bit_map
        .iter()
        .zip(vote_maps.new_bit_map.iter())
        .enumerate()
    {
        for bit_index in 0..8 {
            // remember bits are right to left, so seventh index bit is first byte, leftmost bit
            // 0th index bit is the first bit on the right of the first byte
            let old_bit = (old_byte >> bit_index) & 1;
            let new_bit = (new_byte >> bit_index) & 1;

            if old_bit == new_bit {
                continue;
            }

            #[cfg(feature = "verbose")]
            info!("Bit changed at byte {}, bit {}", byte_index, bit_index);

            let vote = match (old_bit, new_bit) {
                (1, 0) => Vote::Decrement,
                (0, 1) => Vote::Increment,
                _ => return Err(MalformedPayload),
            };

            let food_index = byte_index * 8 + bit_index;
            if let Some(name) = state.remote_bank.food_id_to_name.get(food_index) {
                if !name.is_empty() {
                    votes.push((name.as_str(), vote));
                }
            }
        }
    }

    #[cfg(feature = "verbose")]
    info!("Length of votes: {}", votes.len());
    update_foods(&mut state.redis_connection.clone(), &votes).await?;

    Ok((StatusCode::OK, "Accepted").into_response())
}

pub async fn search_handler(Json(payload): Json<Token>) -> impl IntoResponse {
    (StatusCode::OK, payload.token).into_response()
}
