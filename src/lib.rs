#![deny(
    // missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

//! algoliasearch is an algolia client.
//!
//! # Usage
//! ```no_run
//! # #[macro_use] extern crate serde_derive;
//! # use futures::Future;
//! use algoliasearch::Client;
//!
//! #[derive(Debug, Deserialize, Serialize)]
//! struct User {
//!     name: String,
//!     age: u32,
//! }
//!
//! fn main() -> Result<(), Box<std::error::Error>> {
//!     // read ALGOLIA_APPLICATION_ID and ALGOLIA_API_KEY from env
//!     let index = Client::default().init_index::<User>("users");
//!
//!     let fut = index.search("Bernardo")
//!         .map(|res| {
//!             dbg!(res.hits); // [User { name: "Bernardo", age: 32} ]
//!         })
//!         .map_err(|err| {
//!             eprintln!("{:?}", err);
//!         });
//!     tokio::run(fut);
//!     Ok(())
//! }
//! ```

#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate serde_derive;

#[macro_use]
mod macros;
pub mod client;
pub mod index;

pub use client::Client;
pub use index::{settings, SearchQueryBuilder};

static APPLICATION_ID_HEADER: &str = "x-algolia-application-id";
static API_KEY_HEADER: &str = "x-algolia-api-key";
