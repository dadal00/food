use std::fs;

use prost::Message;

pub mod foods {
    include!(concat!(env!("OUT_DIR"), "/foods.rs"));
}

#[cfg(feature = "payloads")]
mod payloads_lib {
    pub mod payloads {
        include!(concat!(env!("OUT_DIR"), "/payloads.rs"));
    }

    pub mod remote {
        include!("./remote.rs");
    }
}

#[cfg(feature = "payloads")]
pub use payloads_lib::payloads;
#[cfg(feature = "payloads")]
pub use payloads_lib::remote::*;

use foods::Bank;

const BANK_PATH: &str = "../bank.bin";

pub fn get_bank() -> Bank {
    let data = fs::read(BANK_PATH).unwrap();

    Bank::decode(&*data).unwrap()
}

pub fn write_bank(bank: &Bank) {
    fs::write(BANK_PATH, bank.encode_to_vec()).unwrap();
}
