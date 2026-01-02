//! # Meilisearch
//!
//! Search engine to provide all food options directly to the user by proxy.
//!
//!
//!
//! ## Schema
//! - Index for all foods
//! - Fields: name (**string**), votes (**int**), location (**string**, default is "None")
//!
//!
//!
//! ## Cron Job
//! - Every 1-5 minutes, we will run through the Redis hash for foods and sync the votes with Meilisearch
//! - Should just be (name: **string**, votes:**int**)
//!
//!
//!
//! ## Proxy
//! We could expose Meilisearch directly to the frontend. But, we believe the
//! proxy allows for better network communication between the frontend and
//! backend for minimal overhead.
//!
//! Specifically, the user will talk to our backend server which will forward
//! their search queries to Meilisearch and in turn return the response back
//! through our backend.
//!
//! The main drawback we can see would be the extra round trip between our
//! backend and Meilisearch. This is broken down in terms of extra decoding
//! /encoding, network latency of that trip, and any other processing time
//! done on our backend in between.
//!
//! But, given Meilisearch and our backend will run on the same machine, the
//! network latency of that trip is magnitudes smaller than the user trip.
//! In addition, we opt for the protobuf format between our frontend and backend
//! which further minimizes the decoding and encoding. Finally, the extra
//! processing time done by our backend is generaly insignificant compared to
//! user network latency.
//!
//! Now that we have addressed the drawbacks, the main benefits of the proxy
//! lie in the protobuf schema with minimal overhead. Because we can opt for
//! a 1) smaller payload and 2) faster encoding/decoding scheme, we dramatically
//! reduce the work needed on the frontend. This can be especially beneficial
//! when having multiple searches per second.
//!
//!
//!
//! ## Commands
//!
//! Grab relevant keys.
//! ```sh
//! curl -H "Authorization: Bearer $(cat /run/secrets/MEILI_MASTER_KEY)" http://localhost:7700/keys
//! ```
use std::{collections::HashMap, sync::Arc};

use bank::foods::Food;
use meilisearch_sdk::{
    client::Client,
    settings::{MinWordSizeForTypos, Settings, TypoToleranceSettings},
};
use serde::Serialize;

pub const FOOD_INDEX: &str = "foods";
pub const FOOD_ID: &str = "id";
pub const FOOD_NAME: &str = "name";
pub const FOOD_VOTES: &str = "votes";
pub const FOOD_LOCATION: &str = "location";

#[derive(Serialize)]
pub struct MeiliFood {
    pub id: u32,
    pub name: String,
    pub votes: u32,
    pub location: String,
}

pub async fn init_meilisearch(
    meili_url: &str,
    meili_admin_key: &str,
    foods_map: &HashMap<String, Food>,
    food_votes: &HashMap<String, u32>,
) -> Arc<Client> {
    let meili_client = Arc::new(Client::new(meili_url, Some(meili_admin_key)).unwrap());

    upsert_foods(meili_client.clone(), foods_map, food_votes).await;

    meili_client
}

pub async fn upsert_foods(
    meili_client: Arc<Client>,
    foods_map: &HashMap<String, Food>,
    food_votes: &HashMap<String, u32>,
) {
    let meili_foods: Vec<MeiliFood> = foods_map
        .iter()
        .map(|(name, food)| MeiliFood {
            id: food.id,
            name: name.clone(),
            votes: *food_votes.get(name).unwrap_or(&0),
            location: food.location.clone(),
        })
        .collect();

    meili_client
        .index(FOOD_INDEX)
        .set_settings(&init_settings())
        .await
        .unwrap();

    upsert_items(meili_client.clone(), FOOD_INDEX, &meili_foods, FOOD_ID).await;
}

async fn upsert_items<T>(meili_client: Arc<Client>, index_name: &str, items: &[T], id_name: &str)
where
    T: Serialize + Send + Sync,
{
    let _result = meili_client
        .index(index_name)
        .add_or_update(items, Some(id_name))
        .await
        .unwrap()
        .wait_for_completion(&meili_client, None, None)
        .await
        .unwrap();

    #[cfg(feature = "verbose")]
    println!("Meili task result: {:?}", _result);
}

fn init_settings() -> Settings {
    Settings::new()
        .with_ranking_rules([
            "words",
            "typo",
            "proximity",
            "exactness",
            "attribute",
            "sort",
        ])
        .with_distinct_attribute(Some(FOOD_NAME))
        .with_filterable_attributes([FOOD_LOCATION])
        .with_searchable_attributes([FOOD_NAME])
        .with_sortable_attributes([FOOD_VOTES])
        .with_typo_tolerance(TypoToleranceSettings {
            enabled: Some(true),
            disable_on_attributes: None,
            disable_on_words: None,
            min_word_size_for_typos: Some(MinWordSizeForTypos {
                one_typo: Some(5),
                two_typos: Some(9),
            }),
        })
}
