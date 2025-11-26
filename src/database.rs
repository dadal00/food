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
//!
