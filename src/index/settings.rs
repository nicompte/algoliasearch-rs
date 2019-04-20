use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use serde::{
    de::{self, Deserialize, Deserializer, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
use serde_repr::*;

enum_str!(SortFacetValuesBy {
    Count("count"),
    Alpha("alpha"),
});

#[cfg(test)]
mod sort_facet_values_by_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serialize() {
        assert_eq!(
            serde_json::to_string(&SortFacetValuesBy::Count).unwrap(),
            r#""count""#
        );
        assert_eq!(
            serde_json::to_string(&SortFacetValuesBy::Alpha).unwrap(),
            r#""alpha""#
        );
    }

    #[test]
    fn test_deserialize() {
        assert_eq!(
            serde_json::from_str::<SortFacetValuesBy>(r#""count""#).unwrap(),
            SortFacetValuesBy::Count
        );
        assert_eq!(
            serde_json::from_str::<SortFacetValuesBy>(r#""alpha""#).unwrap(),
            SortFacetValuesBy::Alpha
        );
        assert_eq!(
            serde_json::from_str::<SortFacetValuesBy>(r#""unknown""#)
                .unwrap_err()
                .classify(),
            serde_json::error::Category::Data
        );
        assert_eq!(
            serde_json::from_str::<SortFacetValuesBy>(r#""unknown""#)
                .unwrap_err()
                .to_string(),
            "invalid value: unknown SortFacetValuesBy variant: unknown, \
             expected a string for SortFacetValuesBy at line 1 column 9"
        );
    }
}

#[derive(Clone, Debug, PartialEq, Hash)]
/// [https://www.algolia.com/doc/api-reference/api-parameters/typoTolerance/](https://www.algolia.com/doc/api-reference/api-parameters/typoTolerance/)
pub enum TypoTolerance {
    #[allow(missing_docs)]
    Enabled,
    #[allow(missing_docs)]
    Disabled,
    #[allow(missing_docs)]
    Min,
    #[allow(missing_docs)]
    Strict,
}

struct TypoToleranceVisitor;

impl<'de> Visitor<'de> for TypoToleranceVisitor {
    type Value = TypoTolerance;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a bool or String")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value {
            Ok(TypoTolerance::Enabled)
        } else {
            Ok(TypoTolerance::Disabled)
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            "min" => Ok(TypoTolerance::Min),
            "strict" => Ok(TypoTolerance::Strict),
            _ => Err(E::custom(format!(
                r#"expected "min" or "strict", got "{}""#,
                value
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for TypoTolerance {
    fn deserialize<D>(deserializer: D) -> Result<TypoTolerance, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(TypoToleranceVisitor)
    }
}

impl Serialize for TypoTolerance {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TypoTolerance::Enabled => serializer.serialize_bool(true),
            TypoTolerance::Disabled => serializer.serialize_bool(false),
            TypoTolerance::Min => serializer.serialize_str("min"),
            TypoTolerance::Strict => serializer.serialize_str("strict"),
        }
    }
}

#[cfg(test)]
mod typo_tolerance_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serialize() {
        assert_eq!(
            serde_json::to_string(&TypoTolerance::Enabled).unwrap(),
            r#"true"#
        );
        assert_eq!(
            serde_json::to_string(&TypoTolerance::Disabled).unwrap(),
            r#"false"#
        );
        assert_eq!(
            serde_json::to_string(&TypoTolerance::Min).unwrap(),
            r#""min""#
        );
        assert_eq!(
            serde_json::to_string(&TypoTolerance::Strict).unwrap(),
            r#""strict""#
        );
    }

    #[test]
    fn test_deserialize() {
        assert_eq!(
            serde_json::from_str::<TypoTolerance>(r#"true"#).unwrap(),
            TypoTolerance::Enabled
        );
        assert_eq!(
            serde_json::from_str::<TypoTolerance>(r#"false"#).unwrap(),
            TypoTolerance::Disabled
        );
        assert_eq!(
            serde_json::from_str::<TypoTolerance>(r#""min""#).unwrap(),
            TypoTolerance::Min
        );
        assert_eq!(
            serde_json::from_str::<TypoTolerance>(r#""strict""#).unwrap(),
            TypoTolerance::Strict
        );
        assert_eq!(
            serde_json::from_str::<TypoTolerance>(r#""unknown""#)
                .unwrap_err()
                .classify(),
            serde_json::error::Category::Data
        );
        assert_eq!(
            serde_json::from_str::<TypoTolerance>(r#""unknown""#)
                .unwrap_err()
                .to_string(),
            "expected \"min\" or \"strict\", got \"unknown\" at line 1 column 9"
        );
    }
}

/// [https://www.algolia.com/doc/api-reference/api-parameters/removeStopWords/](https://www.algolia.com/doc/api-reference/api-parameters/removeStopWords/)
pub type RemoveStopWords = IgnorePlurals;

#[derive(Clone, Debug, PartialEq, Hash)]
/// [https://www.algolia.com/doc/api-reference/api-parameters/ignorePlurals/](https://www.algolia.com/doc/api-reference/api-parameters/ignorePlurals/)
pub enum IgnorePlurals {
    #[allow(missing_docs)]
    Enabled,
    #[allow(missing_docs)]
    Disabled,
    #[allow(missing_docs)]
    Languages(Vec<String>),
}

struct IgnorePluralsVisitor;

impl<'de> Visitor<'de> for IgnorePluralsVisitor {
    type Value = IgnorePlurals;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a bool or a list of ISO codes")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value {
            Ok(IgnorePlurals::Enabled)
        } else {
            Ok(IgnorePlurals::Disabled)
        }
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut values = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(value) = seq.next_element()? {
            values.push(value);
        }
        Ok(IgnorePlurals::Languages(values))
    }
}

impl<'de> Deserialize<'de> for IgnorePlurals {
    fn deserialize<D>(deserializer: D) -> Result<IgnorePlurals, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(IgnorePluralsVisitor)
    }
}

impl Serialize for IgnorePlurals {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            IgnorePlurals::Enabled => serializer.serialize_bool(true),
            IgnorePlurals::Disabled => serializer.serialize_bool(false),
            IgnorePlurals::Languages(values) => {
                let mut seq = serializer.serialize_seq(Some(values.len()))?;
                for e in values {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
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
            serde_json::to_string(&IgnorePlurals::Enabled).unwrap(),
            r#"true"#
        );
        assert_eq!(
            serde_json::to_string(&IgnorePlurals::Disabled).unwrap(),
            r#"false"#
        );
        assert_eq!(
            serde_json::to_string(&IgnorePlurals::Languages(vec!["fr".to_string()])).unwrap(),
            r#"["fr"]"#
        );
    }

    #[test]
    fn test_deserialize() {
        assert_eq!(
            serde_json::from_str::<IgnorePlurals>(r#"true"#).unwrap(),
            IgnorePlurals::Enabled
        );
        assert_eq!(
            serde_json::from_str::<IgnorePlurals>(r#"false"#).unwrap(),
            IgnorePlurals::Disabled
        );
        assert_eq!(
            serde_json::from_str::<IgnorePlurals>(r#"["fr", "en"]"#).unwrap(),
            IgnorePlurals::Languages(vec!["fr".to_string(), "en".to_string()])
        );
        assert_eq!(
            serde_json::from_str::<IgnorePlurals>(r#""unknown""#)
                .unwrap_err()
                .classify(),
            serde_json::error::Category::Data
        );
        assert_eq!(
            serde_json::from_str::<IgnorePlurals>(r#""unknown""#)
                .unwrap_err()
                .to_string(),
            "invalid type: string \"unknown\", expected a bool or a list of ISO codes at line 1 column 9"
        );
    }
}

/// [https://www.algolia.com/doc/api-reference/api-parameters/queryType/](https://www.algolia.com/doc/api-reference/api-parameters/queryType/)
enum_str!(QueryType {
    PrefixLast("prefixLast"),
    PrefixAll("prefixAll"),
    PrefixNone("prefixNone"),
});

#[cfg(test)]
mod query_type_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serialize() {
        assert_eq!(
            serde_json::to_string(&QueryType::PrefixLast).unwrap(),
            r#""prefixLast""#
        );
        assert_eq!(
            serde_json::to_string(&QueryType::PrefixAll).unwrap(),
            r#""prefixAll""#
        );
        assert_eq!(
            serde_json::to_string(&QueryType::PrefixNone).unwrap(),
            r#""prefixNone""#
        );
    }

    #[test]
    fn test_deserialize() {
        assert_eq!(
            serde_json::from_str::<QueryType>(r#""prefixLast""#).unwrap(),
            QueryType::PrefixLast
        );
        assert_eq!(
            serde_json::from_str::<QueryType>(r#""prefixAll""#).unwrap(),
            QueryType::PrefixAll
        );
        assert_eq!(
            serde_json::from_str::<QueryType>(r#""prefixNone""#).unwrap(),
            QueryType::PrefixNone
        );
        assert_eq!(
            serde_json::from_str::<QueryType>(r#""unknown""#)
                .unwrap_err()
                .classify(),
            serde_json::error::Category::Data
        );
        assert_eq!(
            serde_json::from_str::<QueryType>(r#""unknown""#)
                .unwrap_err()
                .to_string(),
            "invalid value: unknown QueryType variant: unknown, \
             expected a string for QueryType at line 1 column 9"
        );
    }
}

enum_str!(RemoveWordsIfNoResults {
    None("none"),
    LastWords("lastWords"),
    FirstWords("firstWords"),
    AllOptions("allOptions"),
});

enum_str!(ExactOnSingleWordQuery {
    Attribute("attribute"),
    None("none"),
    Word("word"),
});

enum_str!(AlternativesAsExact {
    IgnorePlurals("ignorePlurals"),
    SingleWordSynonym("singleWordSynonym"),
    MultiWordsSynonym("multiWordsSynonym"),
});

#[derive(Clone, Debug, Serialize_repr, Deserialize_repr, PartialEq, Hash)]
#[repr(u8)]
/// [https://www.algolia.com/doc/api-reference/api-parameters/distinct/](https://www.algolia.com/doc/api-reference/api-parameters/distinct/)
pub enum Distinct {
    #[allow(missing_docs)]
    Zero = 0,
    #[allow(missing_docs)]
    One = 1,
    #[allow(missing_docs)]
    Two = 2,
    #[allow(missing_docs)]
    Three = 3,
}

#[derive(Clone, Debug, Serialize_repr, Deserialize_repr, PartialEq, Hash)]
#[repr(u8)]
/// [https://www.algolia.com/doc/api-reference/api-parameters/minProximity/](https://www.algolia.com/doc/api-reference/api-parameters/minProximity/)
pub enum MinProximity {
    #[allow(missing_docs)]
    One = 1,
    #[allow(missing_docs)]
    Two = 2,
    #[allow(missing_docs)]
    Three = 3,
    #[allow(missing_docs)]
    Four = 4,
    #[allow(missing_docs)]
    Five = 5,
    #[allow(missing_docs)]
    Six = 6,
    #[allow(missing_docs)]
    Seven = 7,
}

#[derive(Clone, Builder, Debug, Default, Deserialize, Serialize)]
#[builder(default)]
#[serde(rename_all = "camelCase")]
/// Index settings.
pub struct IndexSettings {
    // attributes
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/searchableAttributes/](https://www.algolia.com/doc/api-reference/api-parameters/searchableAttributes/)
    pub searchable_attributes: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/attributesForFaceting/](https://www.algolia.com/doc/api-reference/api-parameters/attributesForFaceting/)
    pub attributes_for_facetting: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/unretrievableAttributes/](https://www.algolia.com/doc/api-reference/api-parameters/unretrievableAttributes/)
    pub unretrievable_attributes: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/attributesToRetrieve/](https://www.algolia.com/doc/api-reference/api-parameters/attributesToRetrieve/)
    pub attributes_to_retrieve: Option<Vec<String>>,

    // ranking
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/ranking/](https://www.algolia.com/doc/api-reference/api-parameters/ranking/)
    pub ranking: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/customRanking/](https://www.algolia.com/doc/api-reference/api-parameters/customRanking/)
    pub custom_ranking: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/replicas/](https://www.algolia.com/doc/api-reference/api-parameters/replicas/)
    pub replicas: Option<Vec<String>>,

    // faceting
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/maxValuesPerFacet/](https://www.algolia.com/doc/api-reference/api-parameters/maxValuesPerFacet/)
    pub max_values_per_facet: Option<u64>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/sortFacetValuesBy/](https://www.algolia.com/doc/api-reference/api-parameters/sortFacetValuesBy/)
    pub sort_facet_values_by: Option<SortFacetValuesBy>,
    // highlighting-snippeting
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/attributesToHighlight/](https://www.algolia.com/doc/api-reference/api-parameters/attributesToHighlight/)
    pub attributes_to_highlight: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/attributesToSnippet/](https://www.algolia.com/doc/api-reference/api-parameters/attributesToSnippet/)
    pub attributes_to_snippet: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/highlightPreTag/](https://www.algolia.com/doc/api-reference/api-parameters/highlightPreTag/)
    pub highlight_pre_tag: Option<String>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/highlightPostTag/](https://www.algolia.com/doc/api-reference/api-parameters/highlightPostTag/)
    pub highlight_post_tag: Option<String>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/snippetEllipsisText/](https://www.algolia.com/doc/api-reference/api-parameters/snippetEllipsisText/)
    pub snippet_ellipsis_text: Option<String>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/restrictHighlightAndSnippetArrays/](https://www.algolia.com/doc/api-reference/api-parameters/restrictHighlightAndSnippetArrays/)
    pub restrict_highlight_and_snippet_arrays: Option<bool>,

    // pagination
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/hitsPerPage/](https://www.algolia.com/doc/api-reference/api-parameters/hitsPerPage/)
    pub hits_per_page: Option<u64>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/paginationLimitedTo/](https://www.algolia.com/doc/api-reference/api-parameters/paginationLimitedTo/)
    pub pagination_limited_to: Option<u64>,

    // typos
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "minWordSizefor1Typo")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/minWordSizefor1Typo/](https://www.algolia.com/doc/api-reference/api-parameters/minWordSizefor1Typo/)
    pub min_word_sizefor_1_typo: Option<u64>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "minWordSizefor2Typo")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/minWordSizefor2Typos/](https://www.algolia.com/doc/api-reference/api-parameters/minWordSizefor2Typos/)
    pub min_word_sizefor_2_typos: Option<u64>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/typoTolerance/](https://www.algolia.com/doc/api-reference/api-parameters/typoTolerance/)
    pub typo_tolerance: Option<TypoTolerance>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/allowTyposOnNumericTokens/](https://www.algolia.com/doc/api-reference/api-parameters/allowTyposOnNumericTokens/)
    pub allow_typos_on_numeric_tokens: Option<bool>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/disableTypoToleranceOnAttributes/](https://www.algolia.com/doc/api-reference/api-parameters/disableTypoToleranceOnAttributes/)
    pub disable_typo_tolerance_on_attributes: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/disableTypoToleranceOnWords/](https://www.algolia.com/doc/api-reference/api-parameters/disableTypoToleranceOnWords/)
    pub disable_typo_tolerance_on_words: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/separatorsToIndex/](https://www.algolia.com/doc/api-reference/api-parameters/separatorsToIndex/)
    pub separators_to_index: Option<String>,

    // languages
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/ignorePlurals/](https://www.algolia.com/doc/api-reference/api-parameters/ignorePlurals/)
    pub ignore_plurals: Option<IgnorePlurals>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/removeStopWords/](https://www.algolia.com/doc/api-reference/api-parameters/removeStopWords/)
    pub remove_stop_words: Option<RemoveStopWords>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/camelCaseAttributes/](https://www.algolia.com/doc/api-reference/api-parameters/camelCaseAttributes/)
    pub camel_case_attributes: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/decompoundedAttributes/](https://www.algolia.com/doc/api-reference/api-parameters/decompoundedAttributes/)
    pub decompounded_attributes: Option<HashMap<String, Vec<String>>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/keepDiacriticsOnCharacters/](https://www.algolia.com/doc/api-reference/api-parameters/keepDiacriticsOnCharacters/)
    pub keep_diacritics_on_characters: Option<String>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/queryLanguages/](https://www.algolia.com/doc/api-reference/api-parameters/queryLanguages/)
    pub query_languages: Option<Vec<String>>,

    // query-strategy
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/queryType/](https://www.algolia.com/doc/api-reference/api-parameters/queryType/)
    pub query_type: Option<String>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/removeWordsIfNoResults/](https://www.algolia.com/doc/api-reference/api-parameters/removeWordsIfNoResults/)
    pub remove_words_if_no_results: Option<RemoveWordsIfNoResults>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/advancedSyntax/](https://www.algolia.com/doc/api-reference/api-parameters/advancedSyntax/)
    pub advanced_syntax: Option<bool>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/optionalWords/](https://www.algolia.com/doc/api-reference/api-parameters/optionalWords/)
    pub optional_words: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/disablePrefixOnAttributes/](https://www.algolia.com/doc/api-reference/api-parameters/disablePrefixOnAttributes/)
    pub disable_prefix_on_attributes: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/disableExactOnAttributes/](https://www.algolia.com/doc/api-reference/api-parameters/disableExactOnAttributes/)
    pub disable_exact_on_attributes: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/exactOnSingleWordQuery/](https://www.algolia.com/doc/api-reference/api-parameters/exactOnSingleWordQuery/)
    pub exact_on_single_word_query: Option<ExactOnSingleWordQuery>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/alternativesAsExact/](https://www.algolia.com/doc/api-reference/api-parameters/alternativesAsExact/)
    pub alternatives_as_exact: Option<HashSet<AlternativesAsExact>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/enableRules/](https://www.algolia.com/doc/api-reference/api-parameters/enableRules/)
    pub enable_rules: Option<bool>,

    // performance
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/numericAttributesForFiltering/](https://www.algolia.com/doc/api-reference/api-parameters/numericAttributesForFiltering/)
    pub numeric_attributes_for_filtering: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/allowCompressionOfIntegerArray/](https://www.algolia.com/doc/api-reference/api-parameters/allowCompressionOfIntegerArray/)
    pub allow_compression_of_integer_array: Option<bool>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    // advanced
    /// [https://www.algolia.com/doc/api-reference/api-parameters/attributeForDistinct/](https://www.algolia.com/doc/api-reference/api-parameters/attributeForDistinct/)
    pub attribute_for_distinct: Option<String>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/distinct/](https://www.algolia.com/doc/api-reference/api-parameters/distinct/)
    pub distinct: Option<Distinct>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/replaceSynonymsInHighlight/](https://www.algolia.com/doc/api-reference/api-parameters/replaceSynonymsInHighlight/)
    pub replace_synonyms_in_highlight: Option<bool>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/minProximity/](https://www.algolia.com/doc/api-reference/api-parameters/minProximity/)
    pub min_proximity: Option<MinProximity>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/responseFields/](https://www.algolia.com/doc/api-reference/api-parameters/responseFields/)
    pub response_fields: Option<Vec<String>>,
    #[builder(setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// [https://www.algolia.com/doc/api-reference/api-parameters/maxFacetHits/](https://www.algolia.com/doc/api-reference/api-parameters/maxFacetHits/)
    pub max_facet_hits: Option<u64>,
}
