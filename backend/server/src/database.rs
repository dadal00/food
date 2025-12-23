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
use once_cell::sync::Lazy;
use redis::{
    Client, Script,
    aio::{ConnectionManager, ConnectionManagerConfig},
};

const FOODS_HASH: &str = "foods";

static FOODS_INIT_SCRIPT: Lazy<Script> = Lazy::new(|| {
    Script::new(
        r#"
        local key = KEYS[1]

        for i = 1, #ARGV, 2 do
            local field = ARGV[i]
            local value = ARGV[i + 1]
            if redis.call("HEXISTS", key, field) == 0 then
                redis.call("HSET", key, field, value)
            end
        end
    "#,
    )
});

pub async fn init_redis(redis_url: &str, bank: &Bank) -> ConnectionManager {
    let config = ConnectionManagerConfig::new()
        .set_number_of_retries(1)
        .set_connection_timeout(Some(Duration::from_millis(100)));

    let client = Client::open(redis_url).unwrap();
    let mut connection_manager = client
        .get_connection_manager_with_config(config)
        .await
        .unwrap();

    // using a script instead of hset_multiple to avoid overwriting existing values
    let _: () = FOODS_INIT_SCRIPT
        .key(FOODS_HASH)
        .arg(map_keys_to_zero_vec(&bank.foods))
        .invoke_async(&mut connection_manager)
        .await
        .unwrap();

    connection_manager
}

fn map_keys_to_zero_vec<T>(map: &HashMap<String, T>) -> Vec<(&str, String)> {
    map.keys().map(|k| (k.as_str(), "0".to_string())).collect()
}
