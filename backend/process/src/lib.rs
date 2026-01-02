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
use std::collections::hash_map::Entry;

use chrono::{Duration, NaiveDate};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

pub mod models;
pub mod utils;

use bank::{
    foods::{Bank, Food},
    get_bank, write_bank,
};
use models::{ENDPOINT, Response};
use utils::{build_payload, format, reset_locations, sanitize, sanitize_bank, today};

pub async fn load_foods(days_before: u32, days_after: u32) {
    let mut bank = get_bank();
    sanitize_bank(&mut bank);
    reset_locations(&mut bank);

    println!("Loaded Foods: {}", bank.foods.len());
    println!("Loaded Locations: {}\n", bank.locations.len());

    let (new_items, new_locations) = fetch_foods_range(&mut bank, days_before, days_after).await;

    if new_items == 0 && new_locations == 0 {
        println!("No new items or locations found. Exiting.");
    } else {
        println!("Total New Items: {}", new_items);
        println!("Total New Locations: {}\n", new_locations);

        println!("Item Verification: {}", bank.foods.len());
        println!("Location Verification: {}", bank.locations.len());
    }

    sanitize_bank(&mut bank);
    write_bank(&bank);
}

async fn fetch_foods(bank: &mut Bank, client: &Client, date: NaiveDate) -> (usize, usize) {
    let payload = build_payload(&format(date));
    let res = client.post(ENDPOINT).json(&payload).send().await.unwrap();

    #[cfg(feature = "verbose")]
    println!("Status: {}\n", res.status());

    let json_string = res.text().await.unwrap();
    let json: Response = serde_json::from_str(&json_string).unwrap();

    let mut new_locations = 0;
    let mut new_items = 0;

    let mut location = String::new();
    let is_today = date == today();

    for court in json.data.dining_courts {
        let sanitized_location = sanitize(&court.formal_name);

        if sanitized_location.is_empty() {
            continue;
        }

        match bank.locations.entry(sanitized_location.clone()) {
            Entry::Vacant(entry) => {
                #[cfg(feature = "verbose")]
                println!("New location! {}", entry.key());
                entry.insert(bank.next_location_id);

                bank.next_location_id += 1;
                new_locations += 1;
            }
            Entry::Occupied(_) => {}
        }

        if is_today {
            location = sanitized_location.clone();
        }

        for meal in court.daily_menu.meals {
            for station in meal.stations {
                for item_shell in station.items {
                    let sanitized_food = sanitize(&item_shell.item.name);

                    if sanitized_food.is_empty() {
                        continue;
                    }

                    match bank.foods.entry(sanitized_food) {
                        Entry::Vacant(entry) => {
                            #[cfg(feature = "verbose")]
                            println!("New item! {}", entry.key());

                            entry.insert(Food {
                                id: bank.next_food_id,
                                location: location.clone(),
                            });

                            bank.next_food_id += 1;
                            new_items += 1;
                        }
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().location = location.clone();
                        }
                    }
                }
            }
        }
    }

    (new_items, new_locations)
}

async fn fetch_foods_range(bank: &mut Bank, days_before: u32, days_after: u32) -> (usize, usize) {
    let today = today();
    let client = Client::new();

    let pb = ProgressBar::new((days_before + days_after + 1) as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
        )
        .unwrap()
        .progress_chars("=> "),
    );

    let mut new_locations = 0;
    let mut new_items = 0;

    for offset in -(days_before as i32)..=(days_after as i32) {
        let date = today + Duration::days(offset as i64);
        pb.set_message(format!("Fetching {}", date));

        let (fetched_items, fetched_locations) = fetch_foods(bank, &client, date).await;

        new_items += fetched_items;
        new_locations += fetched_locations;

        println!("\n\nNew Items: {}", new_items);
        println!("New Locations: {}\n", new_locations);

        pb.inc(1);
    }

    pb.finish_with_message("Done");
    (new_items, new_locations)
}
