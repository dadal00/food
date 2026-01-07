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
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    time::Duration,
};

use bank::foods::Bank;
use once_cell::sync::Lazy;
use redis::{
    Client, Script,
    aio::{ConnectionManager, ConnectionManagerConfig},
};

use crate::error::AppError::{self, InternalError};

const FOODS_HASH: &str = "foods";

pub async fn init_redis(redis_url: &str, bank: &Bank) -> (ConnectionManager, HashMap<String, u32>) {
    let config = ConnectionManagerConfig::new()
        .set_number_of_retries(1)
        .set_connection_timeout(Some(Duration::from_millis(100)));

    let client = Client::open(redis_url).unwrap();
    let mut connection_manager = client
        .get_connection_manager_with_config(config)
        .await
        .unwrap();

    let food_votes = populate_foods(&bank.foods, &mut connection_manager).await;

    (connection_manager, food_votes)
}

fn map_keys_to_zero_vec<T>(map: &HashMap<String, T>) -> Vec<String> {
    map.keys()
        .flat_map(|k| vec![k.clone(), "0".to_string()])
        .collect()
}

static POPULATE_FOODS_SCRIPT: Lazy<Script> = Lazy::new(|| {
    Script::new(
        r#"
        local hash_key = KEYS[1]
        local result = {}

        for i = 1, #ARGV, 2 do
            local food_key = ARGV[i]
            local votes = ARGV[i + 1]

            if redis.call("HEXISTS", hash_key, food_key) == 0 then
                redis.call("HSET", hash_key, food_key, votes)
            end

            table.insert(result, food_key)
            table.insert(result, redis.call("HGET", hash_key, food_key))
        end

        return result
    "#,
    )
});

pub async fn populate_foods<T>(
    foods_map: &HashMap<String, T>,
    connection_manager: &mut ConnectionManager,
) -> HashMap<String, u32> {
    // using a script instead of hset_multiple to avoid overwriting existing values
    let food_votes_vector: Vec<String> = POPULATE_FOODS_SCRIPT
        .key(FOODS_HASH)
        .arg(map_keys_to_zero_vec(foods_map))
        .invoke_async(connection_manager)
        .await
        .unwrap();

    food_votes_vector
        .chunks(2)
        .map(|c| (c[0].clone(), c[1].parse::<u32>().unwrap()))
        .collect()
}

static UPDATE_FOODS_SCRIPT: Lazy<Script> = Lazy::new(|| {
    Script::new(
        r#"
        local hash = KEYS[1]

        for i = 1, #ARGV, 2 do
            local food_key = ARGV[i]
            local amount = tonumber(ARGV[i + 1])
            
            local current = tonumber(redis.call("HGET", hash, food_key))
            
            if current + amount >= 0 then
                redis.call("HINCRBY", hash, food_key, amount)
            end
        end
        "#,
    )
});

#[derive(Copy, Clone)]
pub enum Vote {
    Increment = 1,
    Decrement = -1,
}

impl Display for Vote {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Vote::Increment => 1,
            Vote::Decrement => -1,
        };
        write!(f, "{}", value)
    }
}

pub async fn update_foods(
    connection_manager: &mut ConnectionManager,
    votes: &[(&str, Vote)],
) -> Result<(), AppError> {
    if votes.is_empty() {
        return Ok(());
    }

    let mut arguments = Vec::with_capacity(votes.len() * 2);
    for (food_key, vote) in votes {
        arguments.push(food_key.to_string());
        arguments.push(vote.to_string());
    }

    let _: () = UPDATE_FOODS_SCRIPT
        .key(FOODS_HASH)
        .arg(arguments)
        .invoke_async(connection_manager)
        .await
        .map_err(|e| InternalError(Box::new(e)))?;

    Ok(())
}
