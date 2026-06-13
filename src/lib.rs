//! # supabase-micro
//!
//! A `#![no_std]` Supabase client for embedded devices.
//! Zero heap allocation. Designed for Cortex-M MCUs.
//!
//! Uses `reqwless` for HTTP transport, `heapless` for fixed-capacity
//! collections, and `serde-json-core` for JSON deserialization.
//!
//! ```rust,no_run
//! use supabase_micro::client::SupabaseClient;
//!
//! let client = SupabaseClient::<256>::new(
//!     "https://your-project.supabase.co",
//!     "your-anon-key",
//! );
//!
//! let mut builder = client.from("users");
//! builder.select("*").unwrap();
//! builder.eq("id", "42").unwrap();
//! builder.limit(1).unwrap();
//!
//! // Then execute with reqwless HttpClient:
//! // let resp = transport::reqwless_transport::execute(&mut http, &builder, &mut url_buf, &mut rx_buf).await?;
//! // let (user, _): (User, _) = resp.deserialize()?;
//! ```
#![no_std]

pub mod auth;
pub mod client;
pub mod error;
pub mod postgrest;
pub mod query;
pub mod response;
pub mod transport;

pub use client::SupabaseClient;
pub use error::{Error, PostgrestError};
pub use postgrest::PostgrestBuilder;
pub use query::{Ordering, QueryBuilder};
pub use response::SupabaseResponse;
