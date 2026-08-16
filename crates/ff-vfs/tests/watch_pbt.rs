//! Property-based tests for file watching types.
//!
//! Tests Property 8: watch event types carry valid ResourceUri.

use proptest::prelude::*;

use ff_vfs::uri::ResourceUri;
use ff_vfs::watch::WatchEvent;

/// Strategy that generates valid scheme names: 1–20 chars of [a-z0-9_-].
fn scheme_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_-]{0,19}".prop_filter("scheme must not be empty", |s| !s.is_empty())
}

/// Strategy for generating valid path components (must start with /).
fn path_strategy() -> impl Strategy<Value = String> {
    "/[a-z][a-z0-9/_.-]{1,30}".prop_filter("path must start with /", |p| p.starts_with('/'))
}

/// Strategy for generating valid ResourceUri values.
fn uri_strategy() -> impl Strategy<Value = ResourceUri> {
    (scheme_strategy(), path_strategy()).prop_map(|(scheme, path)| ResourceUri::new(scheme, path))
}

/// Strategy for generating WatchEvent variants with valid URIs.
fn watch_event_strategy() -> impl Strategy<Value = WatchEvent> {
    prop_oneof![
        uri_strategy().prop_map(WatchEvent::Created),
        uri_strategy().prop_map(WatchEvent::Modified),
        uri_strategy().prop_map(WatchEvent::Deleted),
        (uri_strategy(), uri_strategy())
            .prop_map(|(old_uri, new_uri)| WatchEvent::Renamed { old_uri, new_uri }),
    ]
}

/// Helper: extract all URIs from a WatchEvent.
fn extract_uris(event: &WatchEvent) -> Vec<&ResourceUri> {
    match event {
        WatchEvent::Created(uri) => vec![uri],
        WatchEvent::Modified(uri) => vec![uri],
        WatchEvent::Deleted(uri) => vec![uri],
        WatchEvent::Renamed { old_uri, new_uri } => vec![old_uri, new_uri],
        _ => vec![],
    }
}

// Feature: ff-vfs, Property 8: Watch event types carry valid ResourceUri
// **Validates: Requirement 7.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn all_watch_events_carry_valid_resource_uri(event in watch_event_strategy()) {
        let uris = extract_uris(&event);

        // Every event must carry at least one URI
        prop_assert!(!uris.is_empty(), "watch event must carry at least one ResourceUri");

        for uri in uris {
            // Scheme must be non-empty
            prop_assert!(
                !uri.scheme().is_empty(),
                "URI scheme must not be empty, got: {:?}",
                uri
            );

            // Path must be non-empty and start with /
            prop_assert!(
                !uri.path().is_empty(),
                "URI path must not be empty, got: {:?}",
                uri
            );
            prop_assert!(
                uri.path().starts_with('/'),
                "URI path must start with /, got: {:?}",
                uri
            );

            // The URI can successfully round-trip through Display → parse
            let display_str = uri.to_string();
            let parsed = ResourceUri::parse(&display_str);
            prop_assert!(
                parsed.is_ok(),
                "URI must round-trip through Display → parse, failed for: {}",
                display_str
            );
            let parsed_uri = parsed.unwrap();
            prop_assert_eq!(uri.scheme(), parsed_uri.scheme());
            prop_assert_eq!(uri.path(), parsed_uri.path());
        }
    }

    #[test]
    fn watch_event_variants_are_distinguishable(event in watch_event_strategy()) {
        // Verify each variant can be matched and carries meaningful data
        match &event {
            WatchEvent::Created(uri) => {
                prop_assert!(!uri.scheme().is_empty());
                prop_assert!(!uri.path().is_empty());
            }
            WatchEvent::Modified(uri) => {
                prop_assert!(!uri.scheme().is_empty());
                prop_assert!(!uri.path().is_empty());
            }
            WatchEvent::Deleted(uri) => {
                prop_assert!(!uri.scheme().is_empty());
                prop_assert!(!uri.path().is_empty());
            }
            WatchEvent::Renamed { old_uri, new_uri } => {
                prop_assert!(!old_uri.scheme().is_empty());
                prop_assert!(!old_uri.path().is_empty());
                prop_assert!(!new_uri.scheme().is_empty());
                prop_assert!(!new_uri.path().is_empty());
            }
            _ => {
                // Future variants — assert URIs can be extracted
                let uris = extract_uris(&event);
                for uri in uris {
                    prop_assert!(!uri.scheme().is_empty());
                    prop_assert!(!uri.path().is_empty());
                }
            }
        }
    }
}
