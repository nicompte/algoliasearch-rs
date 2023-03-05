# algoliasearch

`algoliasearch` is a (really incomplete) rust implemention of an algolia client.

[![Crates.io](https://img.shields.io/crates/v/algoliasearch.svg)](https://crates.io/crates/algoliasearch)
[![Documentation](https://docs.rs/algoliasearch/badge.svg)](https://docs.rs/algoliasearch)
[![Build Status](https://dev.azure.com/nicompte/algoliasearch-rs/_apis/build/status/nicompte.algoliasearch-rs?branchName=master)](https://dev.azure.com/nicompte/algoliasearch-rs/_build/latest?definitionId=1&branchName=master)

#### usage

```rust
use algoliasearch::Client;
// needs tokio as a dependency,
// tokio = { version = "1", features = ["macros", "rt", "rt-multi-thread"] }
use tokio;

#[derive(Deserialize)]
struct User {
    name: String,
    age: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<Error>> {
    // read ALGOLIA_APPLICATION_ID and ALGOLIA_API_KEY from env
    let index = Client::default().init_index::<User>("users");

    let res = index.search("Bernardo").await?;
    dbg!(res.hits); // [User { name: "Bernardo", age: 32} ]

    Ok(())
}
```

### todo

- Add all the remaining calls
