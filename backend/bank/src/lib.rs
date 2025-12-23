use std::fs;

use prost::Message;
use reqwest::get;

pub mod foods {
    include!(concat!(env!("OUT_DIR"), "/foods.rs"));
}

use foods::Bank;

const BANK_PATH: &str = "../bank.bin";
const REMOTE_BANK_PATH: &str = "https://github.com/dadal00/food/raw/refs/heads/main/bank.bin";

pub fn get_bank() -> Bank {
    let data = fs::read(BANK_PATH).unwrap();

    Bank::decode(&*data).unwrap()
}

pub async fn get_bank_remote() -> Bank {
    let response = get(REMOTE_BANK_PATH).await.unwrap();
    let bytes = response.bytes().await.unwrap();

    Bank::decode(&*bytes).unwrap()
}

pub fn write_bank(bank: &Bank) {
    fs::write(BANK_PATH, bank.encode_to_vec()).unwrap();
}
