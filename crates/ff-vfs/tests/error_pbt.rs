// Feature: virtual-file-system, Property 9: Error format compliance
// Validates: Requirements 1.4, 1.5; Cross-cutting Req 8
//
// Generates all VfsError variants with random strings for uri, operation, scheme, reason fields.
// Asserts that Display output starts with `[vfs]` and length is ≤ 200 characters.

use ff_vfs::VfsError;
use proptest::prelude::*;

/// Short alphanumeric string for operation names (1–20 chars).
fn short_operation() -> impl Strategy<Value = String> {
    "[a-z_]{1,20}"
}

/// Short URI string in vfs format (keeps total length bounded).
fn short_uri() -> impl Strategy<Value = String> {
    "vfs://[a-z]{1,10}/[a-z/]{1,30}"
}

/// Short scheme name (1–10 chars).
fn short_scheme() -> impl Strategy<Value = String> {
    "[a-z_]{1,10}"
}

/// Short reason string (1–30 chars).
fn short_reason() -> impl Strategy<Value = String> {
    "[a-z ]{1,30}"
}

/// Short I/O error message (1–20 chars).
fn short_io_message() -> impl Strategy<Value = String> {
    "[a-z ]{1,20}"
}

/// Strategy that generates all 10 VfsError variants with short random fields.
fn error_variant_strategy() -> impl Strategy<Value = VfsError> {
    prop_oneof![
        (short_uri(), short_operation())
            .prop_map(|(uri, op)| VfsError::NotFound { uri, operation: op }),
        (short_uri(), short_operation())
            .prop_map(|(uri, op)| VfsError::PermissionDenied { uri, operation: op }),
        (short_uri(), short_operation())
            .prop_map(|(uri, op)| VfsError::AlreadyExists { uri, operation: op }),
        (short_uri(), short_operation())
            .prop_map(|(uri, op)| VfsError::NotADirectory { uri, operation: op }),
        (short_operation(), short_scheme()).prop_map(|(op, provider)| {
            VfsError::UnsupportedOperation {
                operation: op,
                provider,
            }
        }),
        (short_uri(), short_reason())
            .prop_map(|(uri, reason)| VfsError::InvalidUri { uri, reason }),
        short_scheme().prop_map(|scheme| VfsError::ProviderUnavailable { scheme }),
        (short_uri(), short_operation(), 1u64..99999u64).prop_map(|(uri, op, ms)| {
            VfsError::Timeout {
                uri,
                operation: op,
                duration_ms: ms,
            }
        }),
        (short_uri(), short_operation(), short_io_message()).prop_map(|(uri, op, msg)| {
            VfsError::Io {
                uri,
                operation: op,
                source: std::io::Error::new(std::io::ErrorKind::Other, msg),
            }
        }),
        short_scheme().prop_map(|scheme| VfsError::DuplicateScheme { scheme }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// **Validates: Requirements 1.4, 1.5**
    ///
    /// Every VfsError variant's Display output must start with `[vfs]` and be at most 200
    /// characters long when generated with reasonable (short) field values.
    #[test]
    fn error_display_starts_with_vfs_prefix_and_within_length(error in error_variant_strategy()) {
        let msg = error.to_string();
        prop_assert!(
            msg.starts_with("[vfs]"),
            "Display output must start with [vfs], got: {msg}"
        );
        prop_assert!(
            msg.len() <= 200,
            "Display output must be ≤ 200 chars, got {} chars: {msg}",
            msg.len()
        );
    }
}
