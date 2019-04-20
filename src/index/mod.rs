use std::{marker::PhantomData, mem, fmt};

use chrono::{DateTime, Utc};
use futures::{Future, Stream};

use reqwest::{
    header::HeaderMap,
    r#async::{Client as AsyncClient, Decoder},
};
use serde::{
    de::{self, Deserialize, Deserializer, DeserializeOwned, Visitor},
    ser::{Serialize, Serializer},
};

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

#[derive(Clone, Debug, PartialEq, Hash)]
/// [https://www.algolia.com/doc/api-reference/api-parameters/aroundRadius/](https://www.algolia.com/doc/api-reference/api-parameters/aroundRadius/)
pub enum AroundRadius {
    #[allow(missing_docs)]
    All,
    #[allow(missing_docs)]
    Radius(u64),
}

struct AroundRadiusVisitor;

impl<'de> Visitor<'de> for AroundRadiusVisitor {
    type Value = AroundRadius;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("all or radius in meters")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value == "all" {
            Ok(AroundRadius::All)
        } else {
            Err(E::custom(format!(
                r#"expected "all", got "{}""#,
                value
            )))
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
    {
        Ok(AroundRadius::Radius(value))
    }
}

impl<'de> Deserialize<'de> for AroundRadius {
    fn deserialize<D>(deserializer: D) -> Result<AroundRadius, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(AroundRadiusVisitor)
    }
}

impl Serialize for AroundRadius {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            AroundRadius::All => serializer.serialize_str("all"),
            AroundRadius::Radius(v) => serializer.serialize_u64(*v),
        }
    }
}

#[cfg(test)]
mod ignore_plurals_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serialize() {
        assert_eq!(
            serde_json::to_string(&AroundRadius::All).unwrap(),
            r#""all""#
        );
        assert_eq!(
            serde_json::to_string(&AroundRadius::Radius(20)).unwrap(),
            r#"20"#
        );
    }

    #[test]
    fn test_deserialize() {
        assert_eq!(
            serde_json::from_str::<AroundRadius>(r#""all""#).unwrap(),
            AroundRadius::All
        );
        assert_eq!(
            serde_json::from_str::<AroundRadius>(r#"20"#).unwrap(),
            AroundRadius::Radius(20)
        );
        assert_eq!(
            serde_json::from_str::<AroundRadius>(r#""unknown""#)
                .unwrap_err()
                .to_string(),
            "expected \"all\", got \"unknown\" at line 1 column 9"
        );
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StringOrVecOfString {
    String(String),
    VecOfString(Vec<String>),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/attributesToRetrieve/](https://www.algolia.com/doc/api-reference/api-parameters/attributesToRetrieve/)
    attributes_to_retrieve: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [](https://www.algolia.com/doc/api-reference/api-parameters/restrictSearchableAttributes/https://www.algolia.com/doc/api-reference/api-parameters/restrictSearchableAttributes/)
    restrict_searchable_attributes: Option<Vec<String>>,

    // filtering
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/filters/](https://www.algolia.com/doc/api-reference/api-parameters/filters/)
    filters: Option<String>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/facetFilters/](https://www.algolia.com/doc/api-reference/api-parameters/facetFilters/)
    facet_filters: Option<Vec<StringOrVecOfString>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/optionalFilters/](https://www.algolia.com/doc/api-reference/api-parameters/optionalFilters/)
    optional_filters: Option<Vec<StringOrVecOfString>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/numericFilters/](https://www.algolia.com/doc/api-reference/api-parameters/numericFilters/)
    numeric_filters: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/tagFilters/](https://www.algolia.com/doc/api-reference/api-parameters/tagFilters/)
    tag_filters: Option<StringOrVecOfString>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/sumOrFiltersScores/](https://www.algolia.com/doc/api-reference/api-parameters/sumOrFiltersScores/)
    sum_or_filters_scores: Option<bool>,

    // faceting
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/facets/](https://www.algolia.com/doc/api-reference/api-parameters/facets/)
    facets: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/maxValuesPerFacet/](https://www.algolia.com/doc/api-reference/api-parameters/maxValuesPerFacet/)
    max_values_per_facet: Option<u64>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/facetingAfterDistinct/](https://www.algolia.com/doc/api-reference/api-parameters/facetingAfterDistinct/)
    faceting_after_distinct: Option<bool>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/sortFacetValuesBy/](https://www.algolia.com/doc/api-reference/api-parameters/sortFacetValuesBy/)
    sort_facet_values_by: Option<crate::settings::SortFacetValuesBy>,

    // highlighting-snippeting
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/attributesToHighlight/](https://www.algolia.com/doc/api-reference/api-parameters/attributesToHighlight/)
    attributes_to_highlight: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/attributesToSnippet/](https://www.algolia.com/doc/api-reference/api-parameters/attributesToSnippet/)
    attributes_to_snippet: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/highlightPreTag/](https://www.algolia.com/doc/api-reference/api-parameters/highlightPreTag/)
    highlight_pre_tag: Option<String>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/highlightPostTag/](https://www.algolia.com/doc/api-reference/api-parameters/highlightPostTag/)
    highlight_post_tag: Option<String>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/snippetEllipsisText/](https://www.algolia.com/doc/api-reference/api-parameters/snippetEllipsisText/)
    snippet_ellipsis_text: Option<String>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/restrictHighlightAndSnippetArrays/](https://www.algolia.com/doc/api-reference/api-parameters/restrictHighlightAndSnippetArrays/)
    restrict_highlight_and_snippet_arrays: Option<bool>,

    // pagination
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/page/](https://www.algolia.com/doc/api-reference/api-parameters/page/)
    page: Option<u64>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/hitsPerPage/](https://www.algolia.com/doc/api-reference/api-parameters/hitsPerPage/)
    hits_per_page: Option<u64>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/offset/](https://www.algolia.com/doc/api-reference/api-parameters/offset/)
    offset: Option<u64>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/length/](https://www.algolia.com/doc/api-reference/api-parameters/length/)
    length: Option<u64>,

    // typos
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "minWordSizefor1Typo")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/minWordSizefor1Typo/](https://www.algolia.com/doc/api-reference/api-parameters/minWordSizefor1Typo/)
    min_word_sizefor_1_typo: Option<u64>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "minWordSizefor2Typo")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/minWordSizefor2Typos/](https://www.algolia.com/doc/api-reference/api-parameters/minWordSizefor2Typos/)
    min_word_sizefor_2_typos: Option<u64>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/typoTolerance/](https://www.algolia.com/doc/api-reference/api-parameters/typoTolerance/)
    typo_tolerance: Option<crate::settings::TypoTolerance>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/allowTyposOnNumericTokens/](https://www.algolia.com/doc/api-reference/api-parameters/allowTyposOnNumericTokens/)
    allow_typos_on_numeric_tokens: Option<bool>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/disableTypoToleranceOnAttributes/](https://www.algolia.com/doc/api-reference/api-parameters/disableTypoToleranceOnAttributes/)
    disable_typo_tolerance_on_attributes: Option<Vec<String>>,

    // geo-search
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/aroundLatLng/](https://www.algolia.com/doc/api-reference/api-parameters/aroundLatLng/)
    around_lat_lng: Option<String>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "aroundLatLngViaIP")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/aroundLatLngViaIP/](https://www.algolia.com/doc/api-reference/api-parameters/aroundLatLngViaIP/)
    around_lat_lng_via_ip: Option<bool>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/aroundRadius/](https://www.algolia.com/doc/api-reference/api-parameters/aroundRadius/)
    around_radius: Option<AroundRadius>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/aroundPrecision/](https://www.algolia.com/doc/api-reference/api-parameters/aroundPrecision/)
    around_precision: Option<u64>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/minimumAroundRadius/](https://www.algolia.com/doc/api-reference/api-parameters/minimumAroundRadius/)
    minimum_around_radius: Option<u64>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/insideBoundingBox/](https://www.algolia.com/doc/api-reference/api-parameters/insideBoundingBox/)
    inside_bounding_box: Option<Vec<f64>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/insidePolygon/](https://www.algolia.com/doc/api-reference/api-parameters/insidePolygon/)
    inside_polygon: Option<Vec<f64>>,

    // languages
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/ignorePlurals/](https://www.algolia.com/doc/api-reference/api-parameters/ignorePlurals/)
    ignore_plurals: Option<crate::settings::IgnorePlurals>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/removeStopWords/](https://www.algolia.com/doc/api-reference/api-parameters/removeStopWords/)
    remove_stop_words: Option<crate::settings::IgnorePlurals>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/queryLanguages/](https://www.algolia.com/doc/api-reference/api-parameters/queryLanguages/)
    query_languages: Option<Vec<String>>,

    // query-strategy
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/queryType/](https://www.algolia.com/doc/api-reference/api-parameters/queryType/)
    query_type: Option<crate::settings::QueryType>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/removeWordsIfNoResults/](https://www.algolia.com/doc/api-reference/api-parameters/removeWordsIfNoResults/)
    remove_words_if_no_results: Option<crate::settings::RemoveWordsIfNoResults>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/advancedSyntax/](https://www.algolia.com/doc/api-reference/api-parameters/advancedSyntax/)
    advanced_syntax: Option<bool>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/optionalWords/](https://www.algolia.com/doc/api-reference/api-parameters/optionalWords/)
    optional_words: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/disableExactOnAttributes/](https://www.algolia.com/doc/api-reference/api-parameters/disableExactOnAttributes/)
    disable_exact_on_attributes: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/exactOnSingleWordQuery/](https://www.algolia.com/doc/api-reference/api-parameters/exactOnSingleWordQuery/)
    exact_on_single_word_query: Option<crate::settings::ExactOnSingleWordQuery>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/alternativesAsExact/](https://www.algolia.com/doc/api-reference/api-parameters/alternativesAsExact/)
    alternatives_as_exact: Option<crate::settings::AlternativesAsExact>,

    // query-rules
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/enableRules/](https://www.algolia.com/doc/api-reference/api-parameters/enableRules/)
    enable_rules: Option<bool>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/ruleContexts/](https://www.algolia.com/doc/api-reference/api-parameters/ruleContexts/)
    rule_contexts: Option<Vec<String>>,

    // personalization
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/enablePersonalization/](https://www.algolia.com/doc/api-reference/api-parameters/enablePersonalization/)
    enable_personalization: Option<bool>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/userToken/](https://www.algolia.com/doc/api-reference/api-parameters/userToken/)
    user_token: Option<String>,

    // advanced
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/distinct/](https://www.algolia.com/doc/api-reference/api-parameters/distinct/)
    distinct: Option<crate::settings::Distinct>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/getRankingInfo/](https://www.algolia.com/doc/api-reference/api-parameters/getRankingInfo/)
    get_ranking_info: Option<bool>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/clickAnalytics/](https://www.algolia.com/doc/api-reference/api-parameters/clickAnalytics/)
    click_analytics: Option<bool>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/analytics/](https://www.algolia.com/doc/api-reference/api-parameters/analytics/)
    analytics: Option<bool>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/analyticsTags/](https://www.algolia.com/doc/api-reference/api-parameters/analyticsTags/)
    analytics_tags: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/synonyms/](https://www.algolia.com/doc/api-reference/api-parameters/synonyms/)
    synonyms: Option<bool>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/replaceSynonymsInHighlight/](https://www.algolia.com/doc/api-reference/api-parameters/replaceSynonymsInHighlight/)
    replace_synonyms_in_highlight: Option<bool>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/minProximity/](https://www.algolia.com/doc/api-reference/api-parameters/minProximity/)
    min_proximity: Option<crate::settings::MinProximity>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/responseFields/](https://www.algolia.com/doc/api-reference/api-parameters/responseFields/)
    response_fields: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/maxFacetHits/](https://www.algolia.com/doc/api-reference/api-parameters/maxFacetHits/)
    max_facet_hits: Option<u64>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/percentileComputation/](https://www.algolia.com/doc/api-reference/api-parameters/percentileComputation/)
    percentile_computation: Option<u64>,
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
    /// Or a [SearchQuery](struct.SearchQuery.html) object, that can be build with the [SearchQueryBuilder](struct.SearchQueryBuilder.html):
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
    ///     .analytics(false)
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
