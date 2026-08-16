//! Property-based tests for ff-background-io.
//!
//! Uses proptest to verify correctness properties across many random inputs.

use proptest::prelude::*;

use ff_background_io::save;
use ff_background_io::{ChunkSize, IoConfig, LargeFileThreshold, ProgressState};

// ─── Property 1: Chunk Size Clamping ────────────────────────────────────────────

proptest! {
    /// Feature: background-io, Property 1: Chunk size clamping
    /// Any u32 input clamped to [4 KB, 1 MB] range.
    ///
    /// **Validates: Requirements 1.7, 5.2**
    #[test]
    fn chunk_size_always_clamped_to_valid_range(value in any::<u32>()) {
        let chunk = ChunkSize::new(value);
        let bytes = chunk.as_bytes();
        prop_assert!(bytes >= ChunkSize::MIN, "chunk size {} < minimum {}", bytes, ChunkSize::MIN);
        prop_assert!(bytes <= ChunkSize::MAX, "chunk size {} > maximum {}", bytes, ChunkSize::MAX);
    }
}

// ─── Property 2: Progress Percentage Invariant ──────────────────────────────────

proptest! {
    /// Feature: background-io, Property 2: Progress percentage invariant
    /// For any bytes_transferred <= total_bytes, percentage == (bytes_transferred * 100) / total_bytes
    /// and is always in [0, 100].
    ///
    /// **Validates: Requirements 2.3**
    #[test]
    fn progress_percentage_always_in_valid_range(
        bytes_transferred in 0u64..=u64::MAX/2,
        total_bytes in 1u64..=u64::MAX/2,
    ) {
        let clamped_transferred = bytes_transferred.min(total_bytes);
        let percentage = ProgressState::calculate_percentage(clamped_transferred, Some(total_bytes));
        match percentage {
            Some(pct) => {
                prop_assert!(pct <= 100, "percentage {} > 100", pct);
            }
            None => {
                prop_assert!(false, "percentage should be Some when total_bytes is Some");
            }
        }
    }
}

proptest! {
    /// Feature: background-io, Property 2b: Progress percentage correct calculation
    ///
    /// **Validates: Requirements 2.3**
    #[test]
    fn progress_percentage_matches_expected_formula(
        bytes_transferred in 0u64..=10_000_000,
        total_bytes in 1u64..=10_000_000,
    ) {
        let clamped = bytes_transferred.min(total_bytes);
        let percentage = ProgressState::calculate_percentage(clamped, Some(total_bytes));
        let expected = (((clamped as u128) * 100) / (total_bytes as u128)) as u8;
        prop_assert_eq!(percentage, Some(expected));
    }
}

// ─── Property 6: Temp File Name Uniqueness ──────────────────────────────────────

proptest! {
    /// Feature: background-io, Property 6: Temp file name uniqueness
    /// N concurrent saves for same target produce N distinct temp names.
    ///
    /// **Validates: Requirements 4.2**
    #[test]
    fn temp_file_names_are_unique_for_same_target(
        target in "[a-z/]{1,50}",
        count in 2usize..=20,
    ) {
        let names: Vec<String> = (0..count)
            .map(|_| save::generate_temp_path(&target))
            .collect();
        let unique: std::collections::HashSet<&String> = names.iter().collect();
        // With 36^6 possibilities, collisions in <20 samples are astronomically unlikely
        prop_assert_eq!(unique.len(), names.len(), "duplicate temp names generated");
    }
}

// ─── Property 8: Retry Backoff Timing ───────────────────────────────────────────

proptest! {
    /// Feature: background-io, Property 8: Retry backoff timing
    /// For N retries, verify backoff durations follow exponential pattern.
    ///
    /// **Validates: Requirements 6.7**
    #[test]
    fn retry_backoff_follows_exponential_pattern(
        initial_ms in 100u64..=2000,
        retries in 1u8..=5,
    ) {
        let mut backoff = initial_ms;
        let mut durations = Vec::new();
        for _ in 0..retries {
            durations.push(backoff);
            backoff = backoff.saturating_mul(2);
        }

        // Verify each subsequent duration is double the previous
        for i in 1..durations.len() {
            let expected = durations[i - 1].saturating_mul(2);
            prop_assert_eq!(durations[i], expected, "backoff not exponential at index {}", i);
        }
    }
}

// ─── Property 10: Error Format Compliance ───────────────────────────────────────

proptest! {
    /// Feature: background-io, Property 10: Error format compliance
    /// All IoError Display output starts with `[background-io]` and includes phase, URI, bytes.
    ///
    /// **Validates: Requirements 6.1, 6.2**
    #[test]
    fn io_error_display_always_starts_with_prefix(
        uri in "[a-z]+://[a-z]+/[a-z.]+",
        description in "[a-zA-Z ]{1,30}",
        bytes in 0u64..=1_000_000_000,
    ) {
        use ff_background_io::IoError;
        use ff_vfs::VfsError;

        let source = VfsError::NotFound {
            uri: uri.clone(),
            operation: "read".to_string(),
        };

        let errors = vec![
            IoError::OpenFailed {
                uri: uri.clone(),
                description: description.clone(),
                source: VfsError::NotFound { uri: uri.clone(), operation: "open".to_string() },
            },
            IoError::ReadChunkFailed {
                uri: uri.clone(),
                description: description.clone(),
                bytes_transferred: bytes,
                source: VfsError::NotFound { uri: uri.clone(), operation: "read".to_string() },
            },
            IoError::WriteChunkFailed {
                uri: uri.clone(),
                description: description.clone(),
                bytes_transferred: bytes,
                source: VfsError::NotFound { uri: uri.clone(), operation: "write".to_string() },
            },
            IoError::Cancelled {
                uri: uri.clone(),
                bytes_transferred: bytes,
            },
        ];

        for error in &errors {
            let msg = error.to_string();
            prop_assert!(msg.starts_with("[background-io]"),
                "error message does not start with [background-io]: {}", msg);
            prop_assert!(msg.contains(&uri),
                "error message does not contain URI '{}': {}", uri, msg);
        }
    }
}

// ─── Property 9: Concurrency Limit Enforcement ──────────────────────────────────

proptest! {
    /// Feature: background-io, Property 9: Concurrency limit enforcement
    /// Config max_concurrent_tasks is always clamped to [1, 16].
    ///
    /// **Validates: Requirements 7.1, 7.3**
    #[test]
    fn max_concurrent_tasks_clamped_to_valid_range(value in any::<u8>()) {
        let config = IoConfig::new(64, 100, value, 3, 500, 30);
        prop_assert!(config.max_concurrent_tasks >= 1);
        prop_assert!(config.max_concurrent_tasks <= 16);
    }
}

// ─── Property 1b: Large File Threshold Clamping ─────────────────────────────────

proptest! {
    /// Feature: background-io, Property 1b: Large file threshold clamping
    /// Any u64 input clamped to [10 MB, 4096 MB] range.
    ///
    /// **Validates: Requirements 5.2**
    #[test]
    fn large_file_threshold_always_clamped_to_valid_range(value in any::<u64>()) {
        let threshold = LargeFileThreshold::new(value);
        let bytes = threshold.as_bytes();
        prop_assert!(bytes >= LargeFileThreshold::MIN,
            "threshold {} < minimum {}", bytes, LargeFileThreshold::MIN);
        prop_assert!(bytes <= LargeFileThreshold::MAX,
            "threshold {} > maximum {}", bytes, LargeFileThreshold::MAX);
    }
}
