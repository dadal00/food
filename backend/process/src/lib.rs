//! # Food Processing
//!
//! Logic behind foods.
//!
//! ## Overall Data Structures
//!
//! In-memory structures:
//! - Meilisearch update structs (name: **string**, location: **string** with default of "None"): Template struct everytime we perform the cron job to update Meilisearch.
//!   Make a copy each time for reuse. Resets the location and updates locations as needed. Can be extended for new items.
//!
//! - Daily location bitmaps (list of **bitmaps**): Template bitmaps representing daily locations for all foods. Each index is a food index. Must be
//!   updated when new food by adding another bit to each bitmap. No copy, just update each cron job. Zero out at the start of each cron job. Also must be
//!   extended by a new bitmap when a new location is added.
//!
//! ### External
//! - External proto file for food name (**string**) in index ordering (implied index): Allows food indexing in bitmaps such as user vote bitmaps and location bitmaps.
//!   Also allows syncing indexing between frontend (assuming dynamic fetch) and backend.
//!   - Loaded in-memory.
//!
//! - External location file for location (**string**) in index ordering (implied index): Used in location indexing in location bitmaps for frontend and backend.
//!   - Loaded in-memory.
//!
//! #### Notes
//! - Only repeated fields in .proto preserves the index ordering, otherwise implied ordering will be lost.
//!
//! ### Redis
//! - Redis hash for users (**string**) to votes (**bitmap**): O(1) lookups to fetch user votes. Used to fetch user votes and atomic operations to
//!   increment/decrement food votes. TTL: 1 year. Make sure to extend bitmap when new food is added.
//!
//! - Redis hash for foods (**string**) to votes (**int**): O(1) atomic operations to increment/decrement food votes. Used to handle
//!   user votes on incrementing or decrementing.
//!
//! ### Meilisearch
//! - Index for all foods (name: **string**, votes: **int**, location: **string**): Allows for user search of foods to find what to vote for.
//!   Votes attribute will be synced every so often with Redis. Votes attribute allows for filtering in search and one less network call.
//!   Location allows for filtering.
//!
//!
//!
//! ## Notes
//! - External proto file can be seen on the frontend IF we perform a fetch such as fetch('data/proto')
//!   assuming that file is in the frontend static directory.
//!
//! - Otherwise frontend clients will NOT be able to see it if we do not dynamically serve it AND
//!   prevent browser caching.
//!
//! - We should use a fallback just in case the proto file was not fetched properly.
//!
//! - Fallback would be when reading a response that has more bits than the length of our proto file,
//!   we ignore those bits and force a new fetch on the proto file.
//!
//! ## Daily Cron Job -- Purdue API
//! 1. Keep a local copy of cron job foods for Meilisearch so we just need to modify this set to update.
//!
//! 2. Mainly, the name as the ID, the location with default "None".
//!
//! 3. Make a copy of that local meilisearch version.
//!
//! 4. Keep a copy of foods by location to send to users. Just a list of bitmaps. Frontend can understand using the other
//!    external proto file. Default is all zeroed out.
//! - Each day zero out this list of bitmaps.
//!
//! 5. Now, process Purdue API response using backend json. For each item shown under a location, update the location
//!    in the meilisearch copy. May have multiple locations. In addition, flip the bit in the respective location bitmap.
//!
//! 5a. If location not in location map, add it as the next available index. Extend location bitmap as well.
//!
//! 6. If the item does not exist in our copy, check if its gibberish. If not gibberish, it is a new item.
//!
//! 7. Add this new item to in-memory map and both proto files by assigning next available index and adding new food name.
//!
//! 8. Create this new food item for meilisearch with name, index, default 0 votes, and location(s).
//! - No need to add to Redis as increment operation assumes 0 if does not exist.
//!
//! 9. Mark all bitmaps to be updated. Some flag to allow Redis user bitmaps to be updated next time we fetch their data.
//!    Just check the length of the Redis bitmap, if its different, extend it. Also, add an extra bit to each location bitmap.
use std::{collections::HashSet, fs};

use prost::Message;
use reqwest::Client;

pub mod foods {
    include!(concat!(env!("OUT_DIR"), "/foods.rs"));
}
pub mod models;
pub mod utils;

use foods::{Bank, Item as ProtoItem};
use models::{ENDPOINT, Response};
use utils::{build_payload, sanitize, sanitize_bank, today_formatted};

pub async fn fetch_foods() {
    let data = fs::read("../bank.bin").unwrap();
    let mut bank: Bank = Bank::decode(&*data).unwrap();

    sanitize_bank(&mut bank);

    let mut seen: HashSet<String> = bank
        .foods
        .iter()
        .chain(bank.locations.iter())
        .map(|item| item.name.clone())
        .collect();

    println!("Loaded Foods: {}", bank.foods.len());
    println!("Loaded Locations: {}\n", bank.locations.len());

    let client = Client::new();
    let payload = build_payload(&today_formatted());
    let res = client.post(ENDPOINT).json(&payload).send().await.unwrap();

    println!("Status: {}\n", res.status());

    let json_string = res.text().await.unwrap();
    let json: Response = serde_json::from_str(&json_string).unwrap();

    let mut new_locations = 0;
    let mut new_items = 0;
    for court in json.data.dining_courts {
        let mut sanitized = sanitize(&court.formal_name);

        if !sanitized.is_empty() && !seen.contains(&sanitized) {
            println!("New location! {}", sanitized);

            bank.locations.push(ProtoItem {
                id: bank.next_location_id,
                name: sanitized.clone(),
            });
            seen.insert(sanitized);

            bank.next_location_id += 1;
            new_locations += 1;
        }

        for meal in court.daily_menu.meals {
            for station in meal.stations {
                for item_shell in station.items {
                    sanitized = sanitize(&item_shell.item.name);

                    if !sanitized.is_empty() && !seen.contains(&sanitized) {
                        println!("New item! {}", sanitized);

                        bank.foods.push(ProtoItem {
                            id: bank.next_food_id,
                            name: sanitized.clone(),
                        });
                        seen.insert(sanitized);

                        bank.next_food_id += 1;
                        new_items += 1;
                    }
                }
            }
        }
    }

    if new_items == 0 && new_locations == 0 {
        println!("No new items or locations found. Exiting.");
        return;
    }

    println!("New Items: {}", new_items);
    println!("New Locations: {}\n", new_locations);

    println!("Item Verification: {}", bank.foods.len());
    println!("Location Verification: {}", bank.locations.len());

    sanitize_bank(&mut bank);
    let encoded_bytes = bank.encode_to_vec();

    fs::write("../bank.bin", encoded_bytes).unwrap();
}
