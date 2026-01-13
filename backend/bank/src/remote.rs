use anyhow::Error;
use prost::{DecodeError, Message, bytes::Buf};
use reqwest::get;

use crate::{foods::Bank, payloads::Votes};

const REMOTE_BANK_PATH: &str = "https://github.com/dadal00/food/raw/refs/heads/main/bank.bin";

pub struct RemoteBank {
    pub bank: Bank,
    pub food_id_to_name: Vec<String>,
    pub location_id_to_name: Vec<String>,
}

pub async fn get_remote_bank() -> Result<RemoteBank, Error> {
    let response = get(REMOTE_BANK_PATH).await?;
    let bytes = response.bytes().await?;

    let bank = Bank::decode(&*bytes)?;

    let mut food_id_to_name: Vec<String> = vec!["".to_string(); (bank.next_food_id) as usize];
    for (key, value) in bank.foods.clone() {
        food_id_to_name[value.id as usize] = key.clone();
    }

    let mut location_id_to_name: Vec<String> =
        vec!["".to_string(); (bank.next_location_id) as usize];
    for (key, value) in bank.locations.clone() {
        location_id_to_name[value as usize] = key.clone();
    }

    Ok(RemoteBank {
        bank,
        food_id_to_name,
        location_id_to_name,
    })
}

pub fn get_votes_from_bytes<B: Buf>(buf: B) -> Result<Votes, DecodeError> {
    Votes::decode(buf)
}
