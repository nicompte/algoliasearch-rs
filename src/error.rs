#[derive(Debug)]
/// Fetch error
pub enum Error {
    /// Http error
    Http(reqwest::Error),
    /// Json serialization/deserialization error
    Json(serde_json::Error),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Http(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Json(err)
    }
}
