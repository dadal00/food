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

use std::{
    collections::{HashMap, HashSet},
    fs,
};

use chrono::prelude::*;
use prost::Message;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

pub mod foods {
    include!(concat!(env!("OUT_DIR"), "/foods.rs"));
}
use foods::{Bank, Item as ProtoItem};

const ENDPOINT: &str = "https://api.hfs.purdue.edu/menus/v3/GraphQL";

const QUERY: &str = r#"
    query getFoodNames($date: Date!) {
        diningCourts {
            formalName
            dailyMenu(date: $date) {
                meals {
                    name
                    stations {
                        name
                        items {
                            item {
                                name
                            }
                        }
                    }
                }
            }
        }
    }
"#;

#[derive(Deserialize)]
struct Response {
    data: Data,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    dining_courts: Vec<DiningCourt>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiningCourt {
    formal_name: String,
    daily_menu: DailyMenu,
}

#[derive(Deserialize)]
struct DailyMenu {
    meals: Vec<Meal>,
}

#[derive(Deserialize)]
struct Meal {
    name: String,
    stations: Vec<Station>,
}

#[derive(Deserialize)]
struct Station {
    name: String,
    items: Vec<ItemShell>,
}

#[derive(Deserialize)]
struct ItemShell {
    item: Item,
}

#[derive(Deserialize)]
struct Item {
    name: String,
}

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

    println!("New Items: {}", new_items);
    println!("New Locations: {}\n", new_locations);

    println!("Item Verification: {}", bank.foods.len());
    println!("Location Verification: {}", bank.locations.len());

    sanitize_bank(&mut bank);
    let encoded_bytes = bank.encode_to_vec();

    fs::write("../bank.bin", encoded_bytes).unwrap();
}

fn sanitize_bank(bank: &mut Bank) {
    sanitize_vec(&mut bank.foods);
    sanitize_vec(&mut bank.locations);

    unique_bank(bank);

    bank.foods.sort_by(|a, b| a.name.cmp(&b.name));
    bank.locations.sort_by(|a, b| a.name.cmp(&b.name));
}

fn unique_bank(bank: &mut Bank) {
    bank.foods = {
        let mut map = HashMap::new();
        for item in bank.foods.drain(..) {
            map.entry(item.name.clone()).or_insert(item);
        }
        map.into_values().collect()
    };

    bank.locations = {
        let mut map = HashMap::new();
        for loc in bank.locations.drain(..) {
            map.entry(loc.name.clone()).or_insert(loc);
        }
        map.into_values().collect()
    };
}

fn build_payload(date: &str) -> serde_json::Value {
    json!({
        "operationName": "getLocationMenu",
        "variables": { "date": date },
        "query": QUERY
    })
}

fn today_formatted() -> String {
    let today = Local::now().date_naive();
    today.format("%Y-%m-%d").to_string()
}

fn sanitize_vec(vector: &mut Vec<ProtoItem>) {
    vector.iter_mut().for_each(|item| {
        item.name = sanitize(&item.name);
    });
}

fn sanitize(input: &str) -> String {
    let replace = Regex::new(r"[_]").unwrap();
    let mut s = replace.replace_all(input, " ").into_owned();

    let clean_re = Regex::new(r"[^A-Za-z0-9- ]").unwrap();
    s = clean_re.replace_all(&s, "").into_owned();

    s = s.trim().to_string();

    let collapse = Regex::new(r" +").unwrap();
    collapse.replace_all(&s, " ").into_owned().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::sanitize;

    #[test]
    fn test_basic() {
        assert_eq!(sanitize("hello_world"), "hello world");
        assert_eq!(sanitize("Rust-lang"), "rust-lang");
        assert_eq!(sanitize("clean-this_text!"), "clean-this text");
    }

    #[test]
    fn test_leading_trailing_spaces() {
        assert_eq!(sanitize("   hello   "), "hello");
        assert_eq!(sanitize("  multiple   spaces  "), "multiple spaces");
    }

    #[test]
    fn test_special_characters() {
        assert_eq!(sanitize("!@#$%^&*()"), "");
        assert_eq!(sanitize("abc123!@#"), "abc123");
    }

    #[test]
    fn test_underscores_and_dashes() {
        assert_eq!(sanitize("hello_world-test"), "hello world-test");
        assert_eq!(sanitize("_start_end_"), "start end");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(sanitize(""), "");
        assert_eq!(sanitize("     "), "");
    }
}
