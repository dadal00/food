use std::collections::HashMap;

use bank::foods::Bank;
use chrono::prelude::*;
use regex::Regex;
use serde_json::json;

use crate::models::QUERY;

pub fn sanitize_bank(bank: &mut Bank) {
    sanitize_keys(&mut bank.foods);
    sanitize_keys(&mut bank.locations);
}

pub fn build_payload(date: &str) -> serde_json::Value {
    json!({
        "operationName": "getLocationMenu",
        "variables": { "date": date },
        "query": QUERY
    })
}

pub fn today_formatted() -> String {
    let today = Local::now().date_naive();
    today.format("%Y-%m-%d").to_string()
}

pub fn sanitize_keys<V>(map: &mut HashMap<String, V>) {
    let new_map: HashMap<String, V> = map.drain().map(|(k, v)| (sanitize(&k), v)).collect();

    *map = new_map;
}

pub fn sanitize(input: &str) -> String {
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
