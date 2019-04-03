# algoliasearch

`algoliasearch` is a (really incomplete) rust implemention of an algolia client.

### usage

```rust
use algoliasearch::Client;

#[derive(Deserialize)]
struct User {
    name: String, 
    age: u32,
}

fn main() -> Result<(), Box<std::error::Error>> {
    // read ALGOLIA_APPLICATION_ID and ALGOLIA_API_KEY from env
    let index = Client::default().init_index::<User>("INDEX_NAME");

    let res = index.search("Bernardo")?;
    dbg!(res.hits); // [User { name: "Bernardo", age: 32 }]

    let element = index.get_object("8888888")?;
    dbg!(res); // User { name: "Bernardo", age: 32 }
}
```

#### async usage

```rust
use algoliasearch::Client;
use futures::Future;

#[derive(Deserialize)]
struct User {
    name: String, 
    age: u32,
}

fn main() {
    // read ALGOLIA_APPLICATION_ID and ALGOLIA_API_KEY from env
    let index = Client::default().init_index::<User>("INDEX_NAME");

    let test = index
        .search_async("Bernardo")
        .map(|res| {
            dbg!(res.hits); // [User { name: "Bernardo", age: 32} ]
        })
        .map_err(|err| println!("error: {:?}", err));
    tokio::run(test);
}
```

### todo

- Add all the remaining calls
- Find how to unify async and sync implementations