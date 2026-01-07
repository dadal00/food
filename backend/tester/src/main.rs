use std::fs;

use bank::payloads::Votes;
use prost::Message;

fn main() {
    let old_bit_map = vec![0b00000000, 0b00000000];

    let new_bit_map = vec![0b10000000, 0b00000000];

    let votes = Votes {
        old_bit_map,
        new_bit_map,
    };

    println!("{:?}", votes.old_bit_map);
    println!("{:?}", votes.new_bit_map);

    fs::write("../test.bin", votes.encode_to_vec()).unwrap();
}
