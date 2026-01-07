use std::fs;

use bank::{get_bank, payloads::Votes};
use prost::Message;

fn main() {
    let bank = get_bank();

    let num_bits = bank.next_food_id;
    let num_bytes = (num_bits + 7) / 8;

    let old_bit_map = vec![0u8; num_bytes as usize];

    let mut new_bit_map = vec![0u8; num_bytes as usize];
    new_bit_map[0] = 0b10000000;

    let votes = Votes {
        old_bit_map,
        new_bit_map,
    };

    println!("Old bitmap length in bytes: {}", votes.old_bit_map.len());
    println!("New bitmap length in bytes: {}", votes.new_bit_map.len());

    fs::write("../test.bin", votes.encode_to_vec()).unwrap();
}
