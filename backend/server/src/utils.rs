use std::sync::Arc;

use axum::body::Bytes;
use bank::{get_votes_from_bytes, payloads::Votes};

use crate::{
    database::Vote,
    error::AppError::{self, MalformedPayload},
    state::State,
};

pub fn get_maps(state: Arc<State>, bytes: Bytes) -> Result<Votes, AppError> {
    let vote_maps = get_votes_from_bytes(bytes).map_err(|_| AppError::MalformedPayload)?;

    if vote_maps.old_bit_map.len() != vote_maps.new_bit_map.len()
        || vote_maps.old_bit_map.len() > state.remote_bank.food_id_to_name.len()
    {
        return Err(AppError::MalformedPayload);
    }

    Ok(vote_maps)
}

fn compare_bits(
    state: Arc<State>,
    votes: &mut Vec<(isize, Vote)>,
    old_byte: u8,
    new_byte: u8,
    byte_index: usize,
) -> Result<(), AppError> {
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
                votes.push((food_index as isize, vote));
            }
        }
    }

    Ok(())
}

pub fn get_votes_from_body(state: Arc<State>, body: Bytes) -> Result<Vec<(isize, Vote)>, AppError> {
    let vote_maps = get_maps(state.clone(), body)?;
    let mut votes = Vec::new();

    for (byte_index, (&old_byte, &new_byte)) in vote_maps
        .old_bit_map
        .iter()
        .zip(vote_maps.new_bit_map.iter())
        .enumerate()
    {
        compare_bits(state.clone(), &mut votes, old_byte, new_byte, byte_index)?;
    }

    Ok(votes)
}
