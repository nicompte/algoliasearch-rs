use std::{marker::PhantomData, mem};

use chrono::{DateTime, Utc};
use futures::{Future, Stream};

use reqwest::{
    header::HeaderMap,
    r#async::{Client as AsyncClient, Decoder},
    Client as SyncClient,
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
    #[builder(setter(into))]
    query: Option<String>,
    #[builder(setter(into))]
    attributes_to_retrieve: Option<Vec<String>>,
    #[builder(setter(into))]
    restrict_searchable_attributes: Option<Vec<String>>,
    #[builder(setter(into))]
    page: Option<u64>,
    #[builder(setter(into))]
    hits_per_page: Option<u64>,
    #[builder(setter(into))]
    offset: Option<u64>,
    #[builder(setter(into))]
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
    ///
    /// This method accepts a [&str](https://doc.rust-lang.org/std/str/index.html):
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use algoliasearch::Client;
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User;
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// let res = index.search("Bernardo")?;
    /// #   Ok(())
    /// # }
    /// ```
    /// Or a SearchQuery object, that can be build with the [SearchQueryBuilder](struct.SearchQueryBuilder.html):
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User;
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// let query = SearchQueryBuilder::default()
    ///     .query("Bernardo".to_string())
    ///     .page(1)
    ///     .build()?;
    /// let res = index.search(query)?;
    /// #   Ok(())
    /// # }
    /// ```
    pub fn search(&self, query: impl Into<SearchQuery>) -> Result<SearchResult<T>, reqwest::Error> {
        let query = query.into();
        let uri = format!("{}/indexes/{}/query", self.base_url, self.index_name);
        let params = serde_urlencoded::to_string(query).expect("failed to encode params");
        let params = &SearchQueryBody { params };
        SyncClient::new()
            .post(&uri)
            .headers(self.get_headers())
            .json(&params)
            .send()?
            .json()
    }
    /// Get an object from the index.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Serialize, Deserialize, Debug)]
    /// # struct User;
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// let object_1 = index.get_object(&"THE_ID", None)?;
    /// dbg!(object_1); // User { name: "Bernardo", age: 32 };
    /// #   Ok(())
    /// # }
    /// ```
    pub fn get_object(
        &self,
        object_id: &str,
        attributes_to_retrieve: Option<&[&str]>,
    ) -> Result<T, reqwest::Error> {
        let uri = format!(
            "{}/indexes/{}/{}",
            self.base_url, self.index_name, object_id
        );
        SyncClient::new()
            .get(&uri)
            .headers(self.get_headers())
            .query(&[(
                "attributes_to_retrieve",
                attributes_to_retrieve.map(|el| el.join(",")),
            )])
            .send()?
            .json()
    }
    /// Add an object to the index.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { name: String, age: u32, };
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// let object_1 = User { name: "Bernardo".into(), age: 32 };
    /// index.add_object(object_1)?;
    /// #   Ok(())
    /// # }
    /// ```
    pub fn add_object(&self, object: T) -> Result<AddObjectResult, reqwest::Error> {
        let uri = format!("{}/1/indexes/{}", self.base_url, self.index_name);
        SyncClient::new()
            .post(&uri)
            .headers(self.get_headers())
            .json(&object)
            .send()?
            .json()
    }
    /// Add several objects to the index.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { name: String, age: u32, };
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// let object_1 = User { name: "Bernardo".into(), age: 32 };
    /// let object_2 = User { name: "Esmeralda".into(), age: 45 };
    /// index.add_objects(&[object_1, object_2])?;
    /// #   Ok(())
    /// # }
    /// ```
    pub fn add_objects(&self, objects: &[T]) -> Result<BatchedOperatioResult, reqwest::Error> {
        let uri = format!("{}/1/indexes/{}/batch", self.base_url, self.index_name);
        let requests = objects.iter().fold(vec![], |mut acc, x| {
            acc.push(BatchedOperationItem {
                action: "addObject".to_string(),
                body: x,
            });
            acc
        });
        let requests = BatchedOperation { requests };
        SyncClient::new()
            .post(&uri)
            .headers(self.get_headers())
            .json(&requests)
            .send()?
            .json()
    }
    /// Add/update an object to the index. The object will be updated if you provide
    /// a `user_id` property, and added otherwise.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User;
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// #   let object_1 = User;
    /// index.update_object(object_1)?;
    /// #   Ok(())
    /// # }
    /// ```
    pub fn update_object(&self, object: T) -> Result<UpdateOperationResult, reqwest::Error> {
        let uri = format!("{}/1/indexes/{}", self.base_url, self.index_name);
        SyncClient::new()
            .put(&uri)
            .headers(self.get_headers())
            .json(&object)
            .send()?
            .json()
    }
    /// Add/update several objects to the index. The objects will be updated if you provide
    /// a `user_id` property, and added otherwise.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User;
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// #   let object_1 = User;
    /// #   let object_2 = User;
    /// index.update_objects(&[object_1, object_2])?;
    /// #   Ok(())
    /// # }
    /// ```
    pub fn update_objects(&self, objects: &[T]) -> Result<BatchedOperatioResult, reqwest::Error> {
        let uri = format!("{}/1/indexes/{}/batch", self.base_url, self.index_name);
        let requests = objects.iter().fold(vec![], |mut acc, x| {
            acc.push(BatchedOperationItem {
                action: "updateObject".to_string(),
                body: x,
            });
            acc
        });
        let requests = BatchedOperation { requests };
        SyncClient::new()
            .post(&uri)
            .headers(self.get_headers())
            .json(&requests)
            .send()?
            .json()
    }
    /// Delete an object from the index.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { object_id: String, };
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// #   let object_1 = User { object_id: "test".into(), };
    /// index.delete_object(&object_1.object_id)?;
    /// #   Ok(())
    /// # }
    /// ```
    pub fn delete_object(&self, object_id: &str) -> Result<DeleteObjectResult, reqwest::Error> {
        let uri = format!(
            "{}/1/indexes/{}/{}",
            self.base_url, self.index_name, object_id
        );
        SyncClient::new()
            .delete(&uri)
            .headers(self.get_headers())
            .send()?
            .json()
    }
    /// Get the index's settings.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User;
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// #   let index = Client::default().init_index::<User>("users");
    /// let settings = index.get_settings()?;
    /// dbg!(settings.hits_per_page); // 20
    /// #   Ok(())
    /// # }
    /// ```
    pub fn get_settings(&self) -> Result<settings::IndexSettings, reqwest::Error> {
        let uri = format!("{}/indexes/{}/settings", self.base_url, self.index_name);
        SyncClient::new()
            .get(&uri)
            .headers(self.get_headers())
            .send()?
            .json()
    }
    /// Set the index's settings.
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
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
    /// index.set_settings(settings, None)?;
    /// #   Ok(())
    /// # }
    /// ```
    pub fn set_settings(
        &self,
        settings: settings::IndexSettings,
        forward_to_replicas: Option<bool>,
    ) -> Result<UpdateOperationResult, reqwest::Error> {
        let forward_to_replicas = forward_to_replicas.unwrap_or(false);
        let uri = format!("{}/indexes/{}/settings", self.base_url, self.index_name);
        SyncClient::new()
            .put(&uri)
            .headers(self.get_headers())
            .json(&settings)
            .query(&[("forwardToReplicas", forward_to_replicas)])
            .send()?
            .json()
    }
    /// Asynchronously search the index.
    /// It can use the same [SearchQueryBuilder](struct.SearchQueryBuilder.html) as [search()](struct.Index.html#method.search):
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use futures::Future;
    /// # use algoliasearch::{Client, SearchQueryBuilder};
    /// # #[derive(Debug, Serialize, Deserialize)]
    /// # struct User;
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// # let index = Client::default().init_index::<User>("users");
    /// let test = index
    ///     .search_async("Bernardo")
    ///     .map(|res| {
    ///        dbg!(res.hits); // [User { name: "Bernardo", age: 32} ]
    ///     })
    ///     .map_err(|err| eprintln!("error: {:?}", err));
    /// tokio::run(test);
    /// # Ok(())
    /// # }
    /// ```
    pub fn search_async(
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
}
