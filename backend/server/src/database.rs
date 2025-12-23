//! # Redis
//!
//! RAM database.  
//!
//! Core purpose is to store and lookup user votes and food vote counts. Also, used for atomic increment/decrements.
//!
//! ## Requirements
//!
//! - Fast lookups
//! - Decently small dataset
//! - Decent number of food-int pairs ~2000 as an estimated n
//! - Max 50,000 usernames (max 100 chars), n food option pairs
//!
//! ## Implementation
//!
//! - Redis hash: 1 big key, then key-value pairs
//! - Compact pairs and fast lookups
//! - For users: string + n-bit bitmap
//! - For foods: string + 32-bit **votes** int
//! - Atomic operations, Redis loads operations into a queue
//! - Estimated memory usage:
//! (32 bytes (bitmap) + 20 bytes (key overhead)) × 50,000 = roughly 2.6 MB
use std::{collections::HashMap, time::Duration};

use bank::foods::Bank;
use redis::{
    Client,
    aio::{ConnectionManager, ConnectionManagerConfig},
};

const FOODS_HASH: &str = "foods";
const LOCATIONS_HASH: &str = "locations";

pub async fn init_redis(redis_url: &str, bank: &Bank) -> ConnectionManager {
    let config = ConnectionManagerConfig::new()
        .set_number_of_retries(1)
        .set_connection_timeout(Some(Duration::from_millis(100)));

    let client = Client::open(redis_url).unwrap();
    let mut connection_manager = client
        .get_connection_manager_with_config(config)
        .await
        .unwrap();

    let food_pairs = map_keys_to_zero_vec(&bank.foods);
    let location_pairs = map_keys_to_zero_vec(&bank.locations);

    let mut pipe = redis::pipe();
    pipe.atomic()
        .hset_multiple(FOODS_HASH, &food_pairs)
        .hset_multiple(LOCATIONS_HASH, &location_pairs);

    let _: () = pipe.query_async(&mut connection_manager).await.unwrap();

    connection_manager
}

fn map_keys_to_zero_vec<T>(map: &HashMap<String, T>) -> Vec<(&str, String)> {
    map.keys().map(|k| (k.as_str(), "0".to_string())).collect()
}
