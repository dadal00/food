use prost::{Message, bytes::Buf};
use reqwest::get;

use crate::{foods::Bank, payloads::Votes};

pub struct RemoteBank {
    pub bank: Bank,
    pub food_id_to_name: Vec<String>,
    pub location_id_to_name: Vec<String>,
}

const REMOTE_BANK_PATH: &str = "https://github.com/dadal00/food/raw/refs/heads/main/bank.bin";

pub async fn get_remote_bank() -> RemoteBank {
    let response = get(REMOTE_BANK_PATH).await.unwrap();
    let bytes = response.bytes().await.unwrap();

    let bank = Bank::decode(&*bytes).unwrap();

    let mut food_id_to_name: Vec<String> = vec!["".to_string(); (bank.next_food_id) as usize];
    for (key, value) in bank.foods.clone() {
        food_id_to_name[value.id as usize] = key.clone();
    }

    let mut location_id_to_name: Vec<String> =
        vec!["".to_string(); (bank.next_location_id) as usize];
    for (key, value) in bank.locations.clone() {
        location_id_to_name[value as usize] = key.clone();
    }

    RemoteBank {
        bank,
        food_id_to_name,
        location_id_to_name,
    }
}

pub fn get_votes_from_bytes<B: Buf>(buf: B) -> Result<Votes, prost::DecodeError> {
    Votes::decode(buf)
}
