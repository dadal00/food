//! Documentation of a Purdue dining court voting platform.  
//!
//! [Publishing](https://www.reddit.com/r/rust/comments/195ao81/publishing_documentation_as_github_page/) docs to GitHub Pages.  
//! [Example Deployment Workflow](https://github.com/dnaka91/advent-of-code/blob/main/.github/workflows/docs.yml#L28)  
//!
//! # Setup
//!
//! View current docs.
//! ```sh
//! cargo doc --open
//! `````
//!
//! Generate docs in `target/doc/packageName/index.html`.
//! ```sh
//! cargo doc
//! `````
//!
//! # General Infrastructure
//! - User goes to VPS public endpoint running Rust reverse proxy
//! - VPS will reverse proxy into the respective container on the server machine
//! - Only 1 layer reverse proxy, not 2 unlike previous iterations
//! - Containers on server machine will talk to each other using internal names
//! - Ensure ports are exposed on the server machine since LAN not public
//!
//! # Preventing Server Overload
//!
//! **Goal**: Prevent malicious actors from going into my network without touching the frontend first. Acts as a first barrier to attacks.
//!
//! - +server.ts file will provide a HMAC signed timestamp in a cookie to the user, lasts 5 minutes
//! - Frontend sets a timer to refresh this cookie every 4 minutes, basically ensuring valid cookie
//! - Lets server file know by appending a header like `X-refresh`
//! - On the reverse proxy, any time a request wants to go to the backend, we check for a valid HMAC cookie
//! - If invalid or missing, we reject the request
//! - If valid, we check if the timestamp is within 200ms or the **search engine debounce time**
//! - If so, reject the request as it is certainly not a real human interaction
//! - If not, allow the request through
//! - Finally, update the HMAC timestamp, preferably at the end of the request life time
//!
//! # Notes
//!
//! ## Redis + Meilisearch
//! In theory, we could use only Meilisearch for everything including user votes and atomic operations. But,
//! the extra overhead for atomic operations and looking up single user data is too high. Meilisearch is fundamentally
//! a search engine, not a database. Thus, if we do not need search operations and only need looking up or
//! changing user/food data, it is excessive especially if many such operations can happen in a second.  
//!
//! Instead, opting for Redis, an in-memory database, gives atomic operations and O(1) lookups without excessive
//! overhead.
//!
//! We do run into an issue of syncing. But, this is a tradeoff we are okay with as eventual consistency is
//! acceptable for this use case. Meilisearch will not give live results but instead be synced at a consistent interval
//! such as every 5 minutes or a minute.
//!
//! ## 11/14/25
//! - 291 unique foods just for todays
//! - Estimated 291 * 7 = ~2000 foods potential estimate

pub mod database;
pub mod foods;
pub mod search;
pub mod user;
