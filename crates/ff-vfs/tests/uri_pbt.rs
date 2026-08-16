//! Property-based tests for ResourceUri parsing, serialization, and validation.

use std::collections::HashMap;

use ff_vfs::{ResourceUri, VfsError};
use proptest::prelude::*;

/// Generate a valid provider name: 1-20 chars, alphanumeric + hyphen + underscore.
/// Must start with alphanumeric to ensure non-empty valid content.
fn valid_provider_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z][a-zA-Z0-9_-]{0,19}")
        .unwrap()
        .prop_filter("provider must not be empty", |s| !s.is_empty())
}

/// Generate a valid path: non-empty, starts with `/`, contains no `?` or `#`.
fn valid_path_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("/[a-zA-Z0-9_./-]{1,50}")
        .unwrap()
        .prop_filter("path must not be just /", |s| s.len() > 1)
}

/// Generate optional query parameters (0-3 key-value pairs).
fn valid_query_strategy() -> impl Strategy<Value = Option<HashMap<String, String>>> {
    prop::option::of(prop::collection::hash_map(
        prop::string::string_regex("[a-zA-Z][a-zA-Z0-9]{0,9}").unwrap(),
        prop::string::string_regex("[a-zA-Z0-9_.-]{1,20}").unwrap(),
        1..=3,
    ))
}

// Feature: virtual-file-system, Property 1: URI round-trip — serialize via Display, parse back via FromStr, assert equality
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// **Validates: Requirements 2.3, 2.9**
    ///
    /// Any ResourceUri constructed from valid components, when serialized via
    /// Display and parsed back via FromStr, must produce an equal URI.
    #[test]
    fn uri_round_trip_display_then_parse(
        provider in valid_provider_strategy(),
        path in valid_path_strategy(),
    ) {
        let original = ResourceUri::new(&provider, &path);
        let serialized = original.to_string();
        let parsed: ResourceUri = serialized.parse().expect("round-trip parse must succeed");
        prop_assert_eq!(&original, &parsed,
            "round-trip failed: '{}' serialized as '{}' but parsed differently", provider, serialized);
    }

    /// **Validates: Requirements 2.3, 2.9**
    ///
    /// URI round-trip including query parameters.
    #[test]
    fn uri_round_trip_with_query(
        provider in valid_provider_strategy(),
        path in valid_path_strategy(),
        query in valid_query_strategy(),
    ) {
        let original = match query {
            Some(q) if !q.is_empty() => ResourceUri::with_query(&provider, &path, q),
            _ => ResourceUri::new(&provider, &path),
        };
        let serialized = original.to_string();
        let parsed: ResourceUri = serialized.parse().expect("round-trip parse must succeed");
        prop_assert_eq!(&original, &parsed,
            "round-trip with query failed for '{}'", serialized);
    }
}

// Feature: virtual-file-system, Property 2: URI validation rejects invalid inputs — generate invalid strings, assert VfsError::InvalidUri
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// **Validates: Requirements 2.4, 2.5**
    ///
    /// Strings without the `vfs://` prefix must always be rejected with InvalidUri.
    #[test]
    fn rejects_strings_without_vfs_prefix(
        s in "[a-z]{0,5}://[a-z]+/[a-z]+"
            .prop_filter("must not start with vfs://", |s| !s.starts_with("vfs://"))
    ) {
        let result = ResourceUri::parse(&s);
        match result {
            Err(VfsError::InvalidUri { .. }) => {} // expected
            other => prop_assert!(false, "expected InvalidUri for '{}', got {:?}", s, other),
        }
    }

    /// **Validates: Requirements 2.4, 2.5**
    ///
    /// URIs with an empty provider must be rejected.
    #[test]
    fn rejects_empty_provider(
        path in "/[a-z]{1,20}"
    ) {
        let uri_str = format!("vfs://{}", path);
        let result = ResourceUri::parse(&uri_str);
        match result {
            Err(VfsError::InvalidUri { .. }) => {} // expected
            other => prop_assert!(false, "expected InvalidUri for '{}', got {:?}", uri_str, other),
        }
    }

    /// **Validates: Requirements 2.4, 2.5**
    ///
    /// URIs with invalid characters in the provider must be rejected.
    #[test]
    fn rejects_invalid_provider_chars(
        // Generate a provider with at least one invalid character (space, dot, @, etc.)
        invalid_char in prop::sample::select(vec![' ', '.', '@', '!', '#', '$', '%', '&', '*']),
        prefix in "[a-z]{1,5}",
        suffix in "[a-z]{1,5}",
    ) {
        let provider = format!("{}{}{}", prefix, invalid_char, suffix);
        let uri_str = format!("vfs://{}/some/path", provider);
        let result = ResourceUri::parse(&uri_str);
        match result {
            Err(VfsError::InvalidUri { .. }) => {} // expected
            other => prop_assert!(false, "expected InvalidUri for '{}', got {:?}", uri_str, other),
        }
    }

    /// **Validates: Requirements 2.4, 2.5**
    ///
    /// URIs with an empty path (just trailing slash) must be rejected.
    #[test]
    fn rejects_empty_path(
        provider in valid_provider_strategy(),
    ) {
        let uri_str = format!("vfs://{}/", provider);
        let result = ResourceUri::parse(&uri_str);
        match result {
            Err(VfsError::InvalidUri { .. }) => {} // expected
            other => prop_assert!(false, "expected InvalidUri for '{}', got {:?}", uri_str, other),
        }
    }

    /// **Validates: Requirements 2.4, 2.5**
    ///
    /// URIs without any path separator must be rejected.
    #[test]
    fn rejects_no_path_separator(
        provider in valid_provider_strategy(),
    ) {
        let uri_str = format!("vfs://{}", provider);
        let result = ResourceUri::parse(&uri_str);
        match result {
            Err(VfsError::InvalidUri { .. }) => {} // expected
            other => prop_assert!(false, "expected InvalidUri for '{}', got {:?}", uri_str, other),
        }
    }
}

// Feature: virtual-file-system, Property 6: bare path default provider — generate paths without `vfs://` prefix, assert scheme == "local"
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// **Validates: Requirements 2.10, 3.8**
    ///
    /// Any bare path (without `vfs://` prefix) passed to `from_path` must
    /// result in a ResourceUri with scheme == "local".
    #[test]
    fn bare_path_defaults_to_local_provider(
        path in "/[a-zA-Z0-9_./-]{1,60}"
    ) {
        let uri = ResourceUri::from_path(&path);
        prop_assert_eq!(uri.scheme(), "local",
            "from_path('{}') should default to 'local' scheme but got '{}'", path, uri.scheme());
        prop_assert_eq!(uri.path(), path.as_str(),
            "from_path('{}') should preserve the path", path);
    }

    /// **Validates: Requirements 2.10, 3.8**
    ///
    /// Bare paths without leading slash get a slash prepended, still default to "local".
    #[test]
    fn bare_relative_path_defaults_to_local_provider(
        path in "[a-zA-Z][a-zA-Z0-9_./]{1,60}"
            .prop_filter("must not start with vfs://", |s| !s.starts_with("vfs://"))
    ) {
        let uri = ResourceUri::from_path(&path);
        prop_assert_eq!(uri.scheme(), "local",
            "from_path('{}') should default to 'local' scheme but got '{}'", path, uri.scheme());
        let expected_path = format!("/{}", path);
        prop_assert_eq!(uri.path(), expected_path.as_str(),
            "from_path('{}') should prepend '/' to relative path", path);
    }
}
