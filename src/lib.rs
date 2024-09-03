use std::{collections::HashSet, time::Duration};

use regex::RegexSet;
use tower_http::cors::CorsLayer;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum AllowedOrigins {
    Any,
    Mirror,
    #[cfg_attr(feature = "serde", serde(untagged))]
    List(SerdeRegexSet),
}

impl From<AllowedOrigins> for tower_http::cors::AllowOrigin {
    fn from(value: AllowedOrigins) -> Self {
        use tower_http::cors::AllowOrigin;
        match value {
            AllowedOrigins::Any => AllowOrigin::any(),
            AllowedOrigins::Mirror => AllowOrigin::mirror_request(),
            AllowedOrigins::List(origins) => AllowOrigin::predicate(move |origin, _parts| {
                origin.to_str().is_ok_and(|origin| origins.is_match(origin))
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum AllowedHeaders {
    Any,
    Mirror,
    #[cfg_attr(feature = "serde", serde(untagged))]
    List(
        #[cfg_attr(feature = "serde", serde(with = "serde_header_name"))] HashSet<http::HeaderName>,
    ),
}

#[cfg(feature = "serde")]
mod serde_header_name {
    use std::collections::HashSet;

    use http::HeaderName;
    use serde::{de, ser::SerializeSeq, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &HashSet<HeaderName>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for header in value {
            seq.serialize_element(header.to_string().as_str())?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashSet<HeaderName>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values: Vec<String> = Deserialize::deserialize(deserializer)?;
        values
            .into_iter()
            .map(HeaderName::try_from)
            .collect::<Result<HashSet<_>, _>>()
            .map_err(de::Error::custom)
    }
}

impl From<AllowedHeaders> for tower_http::cors::AllowHeaders {
    fn from(value: AllowedHeaders) -> Self {
        use tower_http::cors::AllowHeaders;
        match value {
            AllowedHeaders::Any => AllowHeaders::any(),
            AllowedHeaders::Mirror => AllowHeaders::mirror_request(),
            AllowedHeaders::List(allowed_headers) => AllowHeaders::list(allowed_headers),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum AllowedMethods {
    /// Mirror the request method
    Mirror,
    /// Allow a specific list of methods
    #[cfg_attr(feature = "serde", serde(untagged))]
    List(#[cfg_attr(feature = "serde", serde(with = "serde_method"))] HashSet<http::Method>),
}

impl From<AllowedMethods> for tower_http::cors::AllowMethods {
    fn from(value: AllowedMethods) -> Self {
        use tower_http::cors::AllowMethods;
        match value {
            AllowedMethods::Mirror => AllowMethods::mirror_request(),
            AllowedMethods::List(methods) => AllowMethods::list(methods),
        }
    }
}
#[cfg(feature = "serde")]
mod serde_method {
    use std::collections::HashSet;

    use http::Method;
    use serde::{de, ser::SerializeSeq, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &HashSet<Method>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for header in value {
            seq.serialize_element(header.to_string().as_str())?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashSet<Method>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::str::FromStr;
        let values: Vec<String> = Deserialize::deserialize(deserializer)?;
        values
            .into_iter()
            .map(|value| Method::from_str(&value).map_err(de::Error::custom))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum ExposeHeaders {
    /// Expose all headers by responding with `*`
    Any,
    /// Only expose a specific list of headers
    #[cfg_attr(feature = "serde", serde(untagged))]
    List(
        #[cfg_attr(feature = "serde", serde(with = "serde_header_name"))] HashSet<http::HeaderName>,
    ),
}

impl From<ExposeHeaders> for tower_http::cors::ExposeHeaders {
    fn from(value: ExposeHeaders) -> Self {
        match value {
            ExposeHeaders::Any => tower_http::cors::ExposeHeaders::any(),
            ExposeHeaders::List(headers) => tower_http::cors::ExposeHeaders::list(headers),
        }
    }
}

/// A wrapper around `RegexSet` that is serializable with serde
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SerdeRegexSet(
    #[cfg_attr(feature = "serde", serde(with = "serde_regex_set"))] pub RegexSet,
);

impl std::ops::Deref for SerdeRegexSet {
    type Target = RegexSet;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "serde")]
mod serde_regex_set {
    use regex::RegexSet;
    use serde::{de, ser::SerializeSeq, Deserialize, Deserializer, Serializer};
    use std::collections::HashSet;

    pub fn serialize<S>(value: &RegexSet, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(value.len()))?;
        for regex in value.patterns() {
            sequence.serialize_element(regex)?;
        }
        sequence.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<RegexSet, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values: HashSet<String> = Deserialize::deserialize(deserializer)?;
        RegexSet::new(values).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Vary(
    #[cfg_attr(feature = "serde", serde(with = "serde_header_name"))] pub HashSet<http::HeaderName>,
);

impl From<Vary> for tower_http::cors::Vary {
    fn from(value: Vary) -> Self {
        tower_http::cors::Vary::list(value.0)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub struct Config {
    /// Whether to allow credentials in CORS requests
    #[cfg_attr(feature = "serde", serde(default))]
    pub allow_credentials: bool,
    /// Which request headers can be sent in the actual request.
    /// Controls the [`Access-Control-Allow-Headers`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Headers) response header.
    pub allowed_headers: AllowedHeaders,
    /// Controls how to set the [`Access-Control-Allow-Methods`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Methods) response header.
    pub allowed_methods: AllowedMethods,
    /// Controls how to set the [`Access-Control-Allow-Origin`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Origin) response header.
    pub allowed_origins: AllowedOrigins,
    /// If true, include the [`Access-Control-Allow-Private-Network`](https://wicg.github.io/private-network-access/) response header.
    #[cfg_attr(feature = "serde", serde(default))]
    pub allow_private_network: bool,
    /// The maximum age of the CORS request in seconds
    #[cfg_attr(
        feature = "serde",
        serde(
            with = "humantime_serde",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub max_age: Option<Duration>,
    /// Which headers are exposed to the client.
    /// Controls the [`Access-Control-Expose-Headers`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Expose-Headers) response header.
    pub expose_headers: ExposeHeaders,
    /// Which headers to set in the Vary response header
    #[cfg_attr(feature = "serde", serde(default))]
    pub vary: Vary,
}

impl From<Config> for CorsLayer {
    fn from(config: Config) -> Self {
        let mut layer = CorsLayer::new()
            .allow_credentials(config.allow_credentials)
            .allow_headers(config.allowed_headers)
            .allow_methods(config.allowed_methods)
            .allow_origin(config.allowed_origins)
            .allow_private_network(config.allow_private_network)
            .expose_headers(config.expose_headers)
            .vary(config.vary);

        if let Some(max_age) = config.max_age {
            layer = layer.max_age(max_age);
        }

        layer
    }
}

#[cfg(all(feature = "serde", test))]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_roundtrip() {
        let config = Config {
            allow_credentials: true,
            allowed_headers: AllowedHeaders::List(HashSet::from([
                http::header::CONNECTION,
                http::header::AUTHORIZATION,
            ])),
            allowed_methods: AllowedMethods::Mirror,
            allowed_origins: AllowedOrigins::Any,
            allow_private_network: true,
            max_age: Some(Duration::from_secs(3600)),
            expose_headers: ExposeHeaders::Any,
            vary: Vary(HashSet::from([http::HeaderName::from_static("origin")])),
        };
        let serialized = serde_yaml::to_string(&config).unwrap();
        let deserialized: Config = serde_yaml::from_str(&serialized).unwrap();
        // regex::RegexSet does not implement PartialEq
        assert_eq!(config.allow_credentials, deserialized.allow_credentials);
        assert_eq!(config.allowed_headers, deserialized.allowed_headers);
        assert_eq!(config.allowed_methods, deserialized.allowed_methods);
        assert_eq!(
            config.allow_private_network,
            deserialized.allow_private_network
        );
        assert_eq!(config.max_age, deserialized.max_age);
        assert_eq!(config.expose_headers, deserialized.expose_headers);
        assert_eq!(config.vary, deserialized.vary);
    }
}
