use std::{marker::PhantomData, mem};

use chrono::{DateTime, Utc};
use futures::{Future, Stream};

use reqwest::{
    header::HeaderMap,
    r#async::{Client as AsyncClient, Decoder},
};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;

pub mod settings;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Search result
pub struct SearchResult<T> {
    /// Hits
    pub hits: Vec<T>,
    /// Number of hits
    pub nb_hits: u64,
    /// Page
    pub page: u64,
    /// Number of pages
    pub nb_pages: u64,
    /// Number of hits per page
    pub hits_per_page: u64,
    #[serde(rename = "processingTimeMS")]
    /// Processing time (ms)
    pub processing_time_ms: u64,
    /// Is the search exhaustive?
    pub exhaustive_nb_hits: bool,
    /// Query
    pub query: String,
    /// Params
    pub params: String,
}

#[derive(Debug, Serialize, Default, Builder)]
#[builder(default)]
/// algolia search parameters
/// see [https://www.algolia.com/doc/api-reference/search-api-parameters/](https://www.algolia.com/doc/api-reference/search-api-parameters/)
pub struct SearchQuery {
    
    // search
    #[builder(setter(into))]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/query/](https://www.algolia.com/doc/api-reference/api-parameters/query/)
    query: Option<String>,
    
    // attributes
    #[builder(setter(into))]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/attributesToRetrieve/](https://www.algolia.com/doc/api-reference/api-parameters/attributesToRetrieve/)
    attributes_to_retrieve: Option<Vec<String>>,
    #[builder(setter(into))]
    /// [](https://www.algolia.com/doc/api-reference/api-parameters/restrictSearchableAttributes/https://www.algolia.com/doc/api-reference/api-parameters/restrictSearchableAttributes/)
    restrict_searchable_attributes: Option<Vec<String>>,

    // pagination
    #[builder(setter(into))]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/page/](https://www.algolia.com/doc/api-reference/api-parameters/page/)
    page: Option<u64>,
    #[builder(setter(into))]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/hitsPerPage/](https://www.algolia.com/doc/api-reference/api-parameters/hitsPerPage/)
    hits_per_page: Option<u64>,
    #[builder(setter(into))]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/offset/](https://www.algolia.com/doc/api-reference/api-parameters/offset/)
    offset: Option<u64>,
    #[builder(setter(into))]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/length/](https://www.algolia.com/doc/api-reference/api-parameters/length/)
    length: Option<u64>,
}

#[derive(Serialize)]
struct SearchQueryBody {
    params: String,
}

impl From<&str> for SearchQuery {
    fn from(item: &str) -> Self {
        SearchQuery {
            query: Some(item.to_string()),
            ..Default::default()
        }
    }
}

#[derive(Debug)]
/// Fetch error
pub enum FetchError {
    /// Http error
    Http(reqwest::Error),
    /// Json serialization/deserialization error
    Json(serde_json::Error),
}

impl From<reqwest::Error> for FetchError {
    fn from(err: reqwest::Error) -> FetchError {
        FetchError::Http(err)
    }
}

impl From<serde_json::Error> for FetchError {
    fn from(err: serde_json::Error) -> FetchError {
        FetchError::Json(err)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddObjectResult {
    pub created_at: DateTime<Utc>,
    #[serde(rename = "taskID")]
    pub task_id: u64,
    #[serde(rename = "objectID")]
    pub object_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOperationResult {
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "taskID")]
    pub task_id: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteObjectResult {
    pub deleted_at: DateTime<Utc>,
    #[serde(rename = "taskID")]
    pub task_id: u64,
}

#[derive(Serialize)]
struct BatchedOperationItem<T> {
    action: String,
    body: T,
}

#[derive(Serialize)]
struct BatchedOperation<T> {
    requests: Vec<BatchedOperationItem<T>>,
}

#[derive(Debug, Deserialize)]
pub struct BatchedOperatioResult {
    #[serde(rename = "taskID")]
    pub task_id: u64,
    #[serde(rename = "objectIDs")]
    pub object_ids: Vec<String>,
}

#[derive(Debug)]
/// Index
pub struct Index<T> {
    /// Application id
    pub application_id: String,
    /// Index name
    pub index_name: String,
    pub(crate) api_key: String,
    pub(crate) base_url: String,
    pub(crate) index_type: PhantomData<T>,
}

impl<T: DeserializeOwned + Serialize> Index<T> {
    /// Search the index.    
    /// This method accepts a [&str](https://doc.rust-lang.org/std/str/index.html):
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use futures::Future;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Debug, Serialize, Deserialize)]
    /// # struct User;
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// # let index = Client::default().init_index::<User>("users");
    /// let fut = index
    ///     .search("Bernardo")
    ///     .map(|res| {
    ///        dbg!(res.hits); // [User { name: "Bernardo", age: 32} ]
    ///     })
    ///     .map_err(|err| eprintln!("error: {:?}", err));
    /// tokio::run(fut);
    /// # Ok(())
    /// # }
    /// ```
    /// Or a SearchQuery object, that can be build with the [SearchQueryBuilder](struct.SearchQueryBuilder.html):
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use futures::Future;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Debug, Serialize, Deserialize)]
    /// # struct User;
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// let query = SearchQueryBuilder::default()
    ///     .query("Bernardo".to_string())
    ///     .page(1)
    ///     .build()?;
    /// let fut = index.search(query)
    ///     .map(|res| {
    ///        dbg!(res.hits); // [User { name: "Bernardo", age: 32} ]
    ///     })
    ///     .map_err(|err| eprintln!("error: {:?}", err));
    /// tokio::run(fut);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn search(
        &self,
        query: impl Into<SearchQuery>,
    ) -> impl Future<Item = SearchResult<T>, Error = FetchError> {
        let query = query.into();
        let uri = format!("{}/indexes/{}/query", self.base_url, self.index_name);
        let params = serde_urlencoded::to_string(query).expect("failed to encode params");
        let params = &SearchQueryBody { params };
        AsyncClient::new()
            .post(&uri)
            .headers(self.get_headers())
            .json(&params)
            .send()
            .and_then(|mut res| {
                let body = mem::replace(res.body_mut(), Decoder::empty());
                body.concat2()
            })
            .from_err::<FetchError>()
            .and_then(|body| Ok(serde_json::from_slice(&body)?))
            .from_err::<FetchError>()
    }
    // Build authentication headers.
    fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            crate::APPLICATION_ID_HEADER,
            self.application_id.parse().unwrap(),
        );
        headers.insert(crate::API_KEY_HEADER, self.api_key.parse().unwrap());
        headers
    }
    /// Get an object from the index.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use futures::Future;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Serialize, Deserialize, Debug)]
    /// # struct User;
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// let fut = index.get_object(&"THE_ID", None)
    ///     .map(|res| {
    ///         dbg!(&res); // User { name: "Bernardo", age: 32 };
    ///     })
    ///     .map_err(|err| { eprintln!("{:?}", err); });
    /// tokio::run(fut);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn get_object(
        &self,
        object_id: &str,
        attributes_to_retrieve: Option<&[&str]>,
    ) -> impl Future<Item = T, Error = FetchError> {
        let uri = format!(
            "{}/indexes/{}/{}",
            self.base_url, self.index_name, object_id
        );
        AsyncClient::new()
            .get(&uri)
            .headers(self.get_headers())
            .query(&[(
                "attributes_to_retrieve",
                attributes_to_retrieve.map(|el| el.join(",")),
            )])
            .send()
            .and_then(|mut res| {
                let body = mem::replace(res.body_mut(), Decoder::empty());
                body.concat2()
            })
            .from_err::<FetchError>()
            .and_then(|body| Ok(serde_json::from_slice(&body)?))
            .from_err::<FetchError>()
    }
    /// Add an object to the index.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use futures::Future;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { name: String, age: u32, };
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// let object_1 = User { name: "Bernardo".into(), age: 32 };
    /// let fut = index.add_object(object_1)
    ///     .map(|res| { dbg!(&res); })
    ///     .map_err(|err| { eprintln!("{:?}", err); });
    /// tokio::run(fut);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn add_object(&self, object: T) -> impl Future<Item = AddObjectResult, Error = FetchError> {
        let uri = format!("{}/1/indexes/{}", self.base_url, self.index_name);
        AsyncClient::new()
            .post(&uri)
            .headers(self.get_headers())
            .json(&object)
            .send()
            .and_then(|mut res| {
                let body = mem::replace(res.body_mut(), Decoder::empty());
                body.concat2()
            })
            .from_err::<FetchError>()
            .and_then(|body| Ok(serde_json::from_slice(&body)?))
            .from_err::<FetchError>()
    }
    /// Add several objects to the index.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # use futures::Future;
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { name: String, age: u32, };
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// let object_1 = User { name: "Bernardo".into(), age: 32 };
    /// let object_2 = User { name: "Esmeralda".into(), age: 45 };
    /// let fut = index.add_objects(&[object_1, object_2])
    ///     .map(|res| { dbg!(&res); })
    ///     .map_err(|err| { eprintln!("{:?}", err); });
    /// tokio::run(fut);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn add_objects(
        &self,
        objects: &[T],
    ) -> impl Future<Item = BatchedOperatioResult, Error = FetchError> {
        let uri = format!("{}/1/indexes/{}/batch", self.base_url, self.index_name);
        let requests = objects.iter().fold(vec![], |mut acc, x| {
            acc.push(BatchedOperationItem {
                action: "addObject".to_string(),
                body: x,
            });
            acc
        });
        let requests = BatchedOperation { requests };
        AsyncClient::new()
            .post(&uri)
            .headers(self.get_headers())
            .json(&requests)
            .send()
            .and_then(|mut res| {
                let body = mem::replace(res.body_mut(), Decoder::empty());
                body.concat2()
            })
            .from_err::<FetchError>()
            .and_then(|body| Ok(serde_json::from_slice(&body)?))
            .from_err::<FetchError>()
    }
    /// Add/update an object to the index. The object will be updated if you provide
    /// a `user_id` property, and added otherwise.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use futures::Future;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User;
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// #   let object_1 = User;
    /// let fut = index.update_object(object_1)
    ///     .map(|res| { dbg!(&res); })
    ///     .map_err(|err| { eprintln!("{:?}", err); });
    /// tokio::run(fut);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn update_object(
        &self,
        object: T,
    ) -> impl Future<Item = UpdateOperationResult, Error = FetchError> {
        let uri = format!("{}/1/indexes/{}", self.base_url, self.index_name);
        AsyncClient::new()
            .put(&uri)
            .headers(self.get_headers())
            .json(&object)
            .send()
            .and_then(|mut res| {
                let body = mem::replace(res.body_mut(), Decoder::empty());
                body.concat2()
            })
            .from_err::<FetchError>()
            .and_then(|body| Ok(serde_json::from_slice(&body)?))
            .from_err::<FetchError>()
    }
    /// Add/update several objects to the index. The objects will be updated if you provide
    /// a `user_id` property, and added otherwise.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use futures::Future;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User;
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// #   let object_1 = User;
    /// #   let object_2 = User;
    /// let fut = index.update_objects(&[object_1, object_2])
    ///     .map(|res| { dbg!(&res); })
    ///     .map_err(|err| { eprintln!("{:?}", err); });
    /// tokio::run(fut);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn update_objects(
        &self,
        objects: &[T],
    ) -> impl Future<Item = BatchedOperatioResult, Error = FetchError> {
        let uri = format!("{}/1/indexes/{}/batch", self.base_url, self.index_name);
        let requests = objects.iter().fold(vec![], |mut acc, x| {
            acc.push(BatchedOperationItem {
                action: "updateObject".to_string(),
                body: x,
            });
            acc
        });
        let requests = BatchedOperation { requests };
        AsyncClient::new()
            .post(&uri)
            .headers(self.get_headers())
            .json(&requests)
            .send()
            .and_then(|mut res| {
                let body = mem::replace(res.body_mut(), Decoder::empty());
                body.concat2()
            })
            .from_err::<FetchError>()
            .and_then(|body| Ok(serde_json::from_slice(&body)?))
            .from_err::<FetchError>()
    }
    /// Delete an object from the index.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use futures::Future;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { object_id: String, };
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// #   let object_1 = User { object_id: "test".into(), };
    /// let fut = index.delete_object(&object_1.object_id)
    ///     .map(|res| { dbg!(&res); })
    ///     .map_err(|err| { eprintln!("{:?}", err); });
    /// tokio::run(fut);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn delete_object(
        &self,
        object_id: &str,
    ) -> impl Future<Item = DeleteObjectResult, Error = FetchError> {
        let uri = format!(
            "{}/1/indexes/{}/{}",
            self.base_url, self.index_name, object_id
        );
        AsyncClient::new()
            .delete(&uri)
            .headers(self.get_headers())
            .send()
            .and_then(|mut res| {
                let body = mem::replace(res.body_mut(), Decoder::empty());
                body.concat2()
            })
            .from_err::<FetchError>()
            .and_then(|body| Ok(serde_json::from_slice(&body)?))
            .from_err::<FetchError>()
    }
    /// Get the index's settings.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use futures::Future;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User;
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// let fut = index.get_settings()
    ///     .map(|settings| {
    ///         dbg!(&settings.hits_per_page); // 20
    ///     })
    ///     .map_err(|err| { eprintln!("{:?}", err); });
    /// tokio::run(fut);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn get_settings(&self) -> impl Future<Item = settings::IndexSettings, Error = FetchError> {
        let uri = format!("{}/indexes/{}/settings", self.base_url, self.index_name);
        AsyncClient::new()
            .get(&uri)
            .headers(self.get_headers())
            .send()
            .and_then(|mut res| {
                let body = mem::replace(res.body_mut(), Decoder::empty());
                body.concat2()
            })
            .from_err::<FetchError>()
            .and_then(|body| Ok(serde_json::from_slice(&body)?))
            .from_err::<FetchError>()
    }
    /// Set the index's settings.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use futures::Future;
    /// # use algoliasearch::{
    /// #    Client, SearchQueryBuilder,
    /// #    settings::{IndexSettingsBuilder, SortFacetValuesBy}
    /// # };
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User;
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// let settings = IndexSettingsBuilder::default()
    ///     .hits_per_page(30)
    ///     .sort_facet_values_by(SortFacetValuesBy::Count)
    ///     .build()
    ///     .unwrap();
    /// let fut = index.set_settings(settings, None)
    ///     .map(|res| { dbg!(&res); })
    ///     .map_err(|err| { eprintln!("{:?}", err); });
    /// tokio::run(fut);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn set_settings(
        &self,
        settings: settings::IndexSettings,
        forward_to_replicas: Option<bool>,
    ) -> impl Future<Item = UpdateOperationResult, Error = FetchError> {
        let forward_to_replicas = forward_to_replicas.unwrap_or(false);
        let uri = format!("{}/indexes/{}/settings", self.base_url, self.index_name);
        AsyncClient::new()
            .put(&uri)
            .headers(self.get_headers())
            .json(&settings)
            .query(&[("forwardToReplicas", forward_to_replicas)])
            .send()
            .and_then(|mut res| {
                let body = mem::replace(res.body_mut(), Decoder::empty());
                body.concat2()
            })
            .from_err::<FetchError>()
            .and_then(|body| Ok(serde_json::from_slice(&body)?))
            .from_err::<FetchError>()
    }
}
