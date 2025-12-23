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
use std::sync::Arc;

use meilisearch_sdk::client::Client;

pub async fn init_meilisearch(meili_url: &str, meili_admin_key: &str) -> Arc<Client> {
    Arc::new(Client::new(meili_url, Some(meili_admin_key)).unwrap())
}
