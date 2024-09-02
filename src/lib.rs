use std::{collections::HashSet, time::Duration};

use regex::RegexSet;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum AllowedOrigins {
    Any,
    Mirror,
    #[cfg_attr(feature = "serde", serde(untagged))]
    List(SerdeRegexSet),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum AllowedHeaders {
    Any,
    Mirror,
    #[cfg_attr(feature = "serde", serde(untagged))]
    List(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum AllowedMethods {
    /// Mirror the request method
    Mirror,
    /// Allow a specific list of methods
    #[cfg_attr(feature = "serde", serde(untagged))]
    List(HashSet<Method>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum ExposeHeaders {
    /// Expose all headers by responding with `*`
    Any,
    /// Only expose a specific list of headers
    #[cfg_attr(feature = "serde", serde(untagged))]
    List(HashSet<String>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "UPPERCASE"))]
pub enum Method {
    Connect,
    Delete,
    Get,
    Head,
    Options,
    Patch,
    Post,
    Put,
    Trace,
}

impl From<Method> for http::Method {
    fn from(method: Method) -> Self {
        match method {
            Method::Connect => http::Method::CONNECT,
            Method::Delete => http::Method::DELETE,
            Method::Get => http::Method::GET,
            Method::Head => http::Method::HEAD,
            Method::Options => http::Method::OPTIONS,
            Method::Patch => http::Method::PATCH,
            Method::Post => http::Method::POST,
            Method::Put => http::Method::PUT,
            Method::Trace => http::Method::TRACE,
        }
    }
}

/// A wrapper around `RegexSet` that is serializable with serde
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SerdeRegexSet(#[cfg_attr(feature = "serde", serde(with = "serde_regex_set"))] RegexSet);

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
    /// If true, `vary: origin` will be set in the response headers
    #[cfg_attr(feature = "serde", serde(default))]
    pub vary_origin: bool,
}

#[cfg(all(feature = "serde", test))]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_roundtrip() {
        let config = Config {
            allow_credentials: true,
            allowed_headers: AllowedHeaders::List(vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
            ]),
            allowed_methods: AllowedMethods::Mirror,
            allowed_origins: AllowedOrigins::Any,
            allow_private_network: true,
            max_age: Some(Duration::from_secs(3600)),
            expose_headers: ExposeHeaders::Any,
            vary_origin: true,
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
        assert_eq!(config.vary_origin, deserialized.vary_origin);
    }
}
