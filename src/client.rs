use std::{env, marker::PhantomData};

use crate::index;

const ALGOLIA_APPLICATION_ID_VARIABLE: &str = "ALGOLIA_APPLICATION_ID";
const ALGOLIA_API_KEY_VARIABLE: &str = "ALGOLIA_API_KEY";

/// Algolia client
#[derive(Debug)]
pub struct Client {
    application_id: Option<String>,
    api_key: Option<String>,
}

impl Client {
    /// Initialize the client, providing your [APPLICATION_ID](https://www.algolia.com/doc/guides/sending-and-managing-data/send-and-update-your-data/how-to/importing-with-the-api/#application-id)
    /// and your [API_KEY](https://www.algolia.com/doc/guides/sending-and-managing-data/send-and-update-your-data/how-to/importing-with-the-api/#api-key).
    pub fn new(application_id: &str, api_key: &str) -> Client {
        Client {
            application_id: Some(application_id.to_owned()),
            api_key: Some(api_key.to_owned()),
        }
    }
    /// Set your client's [APPLICATION_ID](https://www.algolia.com/doc/guides/sending-and-managing-data/send-and-update-your-data/how-to/importing-with-the-api/#application-id).
    pub fn application_id(mut self, application_id: &str) -> Client {
        self.application_id = Some(application_id.to_owned());
        self
    }
    /// Set you client's [API_KEY](https://www.algolia.com/doc/guides/sending-and-managing-data/send-and-update-your-data/how-to/importing-with-the-api/#api-key)
    pub fn api_key(mut self, api_key: &str) -> Client {
        self.api_key = Some(api_key.to_owned());
        self
    }
    /// Initialize the client index, providing your [INDEX_NAME](#).
    /// ```no_run
    /// # #[macro_use] extern crate serde_derive;
    /// # use algoliasearch::Client;
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User;
    /// # fn main() {
    /// let index = Client::default().init_index::<User>("users");
    /// # }
    /// ```
    pub fn init_index<T>(self, index_name: &str) -> index::Index<T> {
        if self.application_id.is_none() || self.api_key.is_none() {
            panic!("application_id and/or api_key are not initialized");
        }
        index::Index {
            application_id: self.application_id.clone().expect("can't panic"),
            api_key: self.api_key.expect("can't panic"),
            index_name: index_name.to_owned(),
            base_url: format!(
                "https://{}-dsn.algolia.net/1",
                self.application_id.expect("can't panic")
            ),
            index_type: PhantomData,
        }
    }
}

impl Default for Client {
    /// Initialize the client. It will read the environment or .env `ALGOLIA_APPLICATION_ID` and `ALGOLIA_API_KEY` variables to be used as
    /// [APPLICATION_ID](https://www.algolia.com/doc/guides/sending-and-managing-data/send-and-update-your-data/how-to/importing-with-the-api/#application-id)
    /// and [API_KEY](https://www.algolia.com/doc/guides/sending-and-managing-data/send-and-update-your-data/how-to/importing-with-the-api/#api-key).
    fn default() -> Client {
        Client {
            application_id: env::var(ALGOLIA_APPLICATION_ID_VARIABLE).ok(),
            api_key: env::var(ALGOLIA_API_KEY_VARIABLE).ok(),
        }
    }
}

#[cfg(test)]
mod client_tests {
    use super::*;

    struct User;
    #[test]
    #[should_panic(expected = "application_id and/or api_key are not initialized")]
    fn test_missing_application_id() {
        Client::default()
            .api_key("api")
            .init_index::<User>("will fail");
    }
    #[test]
    #[should_panic(expected = "application_id and/or api_key are not initialized")]
    fn test_missing_api_key() {
        Client::default()
            .application_id("application")
            .init_index::<User>("will fail");
    }
    #[test]
    #[should_panic(expected = "application_id and/or api_key are not initialized")]
    fn test_missing_application_id_and_api_key() {
        Client::default().init_index::<User>("will fail");
    }
}
