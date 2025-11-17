//! # Frontend Specifications
//!
//! Client logic/relevant structures.
//!
//! ## Overall Payloads
//!
//! Responses/requests between the frontend and backend.
//!
//! ### Verification
//! Cookies
//! - token: HMAC blob containing timestamp lasting 5 minutes
//! - valid_id: verified UUID v4 string lasting 1 month
//! - invalid_id: unverified UUID v4 string lasting 10 minutes
//!
//! Headers
//! - X-refresh: anything, the header itself is more important to refresh the token every 4 minutes or before the token expiration
//!
//! ### Fetch/Update User Votes
//! - Protobuf, N bits bitmap representing user votes
//! - Just flip the respective bits and it will overwrite whatever we had for the user
//!
//! If no valid_id cookie
//! - Append username string to protobuf
//!
//! If username is valid
//! From backend
//! - 200 message + invalid_id cookie
//!
//! Sending code
//! - Protobuf, 6 digit numeric code as string (keeps leading zeroes) + invalid_id cookie
//!
//! If valid code
//! From backend
//! - remove invalid_id cookie
//! - 200 message + valid_id cookie
//!
//! ### Search/Filter Votes
//! To backend
//! - Protobuf, 4 bits representing which is here, query string, location filter u8 enum, sort by bit flag to indicate direction, paiganation u32 number
//!
//! From backend
//! - Protobuf, bitmap representing which food is here, order in bitmap represents order of foods
//! - list of u32s, each is a vote count
//!
//!
//!
//! ## Flow
//!
//! - Once user tries first vote, ask for purdue username
//! - Send 6-digit numeric verification code to email
//! - If verified, give long-lasting cookie with JWT signed username for future use
//! - If has cookie in future, no need to check for verification, just use username in cookie
//! - Cookie is secure cause we sign using secret and salt
//! - Future requests can just use the encrypted username
//!
//!
//!
//! ## Voting
//!
//! - Search engine provides voting options, hide vote count
//! - Search for food
//! - Debounce search input by 200-500 ms to prevent overload
//! - Semantic search
//! - Thumbs-up or not
//! - Delay pushing user inputs until page refresh **or** arbitrary max thumbs reached
//!
//!
//!
//! ## Verification
//!
//! - Using a +page.server.ts page, check if they have a flagged session cookie (w:id)
//! - Even if malicious actor forges cookie, the backend won't process the vote as it won't be in the database
//! - If not, return the flag to enter a username
//! - If so, mark the flag to NOT do a username
//! - If needs username, the user enters username when submitting a vote
//! - We send back a response with a non-flagged cookie (q:id) holding the session ID
//! - The user is then prompted to enter the 6-digit code
//! - If the code is wrong, we give an error
//! - If the code is right, we'll process the id and overwrite the cookie flag to (w:id)
