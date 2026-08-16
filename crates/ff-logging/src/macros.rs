//! Log macros: `log_trace!`, `log_debug!`, `log_info!`, `log_warn!`, `log_error!`.
//!
//! Zero-cost convenience macros that check the level guard before evaluating
//! format arguments. The level check is an atomic operation — if the record
//! would be filtered, no string formatting or allocation occurs.
//!
//! Each macro automatically captures `module_path!()` for the source module field.

/// Emit a TRACE-level log record.
///
/// The format arguments are only evaluated if TRACE level passes the
/// configured minimum level filter. The formatting closure is never
/// invoked when the level is filtered out, ensuring zero-cost filtering.
///
/// # Examples
///
/// ```rust,ignore
/// ff_logging::log_trace!("Processing item {}", item_id);
/// ```
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        $crate::log_lazy($crate::LogLevel::Trace, module_path!(), || format!($($arg)*))
    };
}

/// Emit a DEBUG-level log record.
///
/// The format arguments are only evaluated if DEBUG level passes the
/// configured minimum level filter. The formatting closure is never
/// invoked when the level is filtered out, ensuring zero-cost filtering.
///
/// # Examples
///
/// ```rust,ignore
/// ff_logging::log_debug!("Cache hit for key: {}", key);
/// ```
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::log_lazy($crate::LogLevel::Debug, module_path!(), || format!($($arg)*))
    };
}

/// Emit an INFO-level log record.
///
/// The format arguments are only evaluated if INFO level passes the
/// configured minimum level filter. The formatting closure is never
/// invoked when the level is filtered out, ensuring zero-cost filtering.
///
/// # Examples
///
/// ```rust,no_run
/// ff_logging::log_info!("Application started successfully");
/// ```
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::log_lazy($crate::LogLevel::Info, module_path!(), || format!($($arg)*))
    };
}

/// Emit a WARN-level log record.
///
/// The format arguments are only evaluated if WARN level passes the
/// configured minimum level filter. The formatting closure is never
/// invoked when the level is filtered out, ensuring zero-cost filtering.
///
/// # Examples
///
/// ```rust,no_run
/// ff_logging::log_warn!("Config value out of range, using default");
/// ```
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::log_lazy($crate::LogLevel::Warn, module_path!(), || format!($($arg)*))
    };
}

/// Emit an ERROR-level log record.
///
/// The format arguments are only evaluated if ERROR level passes the
/// configured minimum level filter. The formatting closure is never
/// invoked when the level is filtered out, ensuring zero-cost filtering.
///
/// # Examples
///
/// ```rust,ignore
/// ff_logging::log_error!("Failed to open file: {}", err);
/// ```
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::log_lazy($crate::LogLevel::Error, module_path!(), || format!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};

    /// Helper type that records whether its `Display` implementation was called.
    struct EvalTracker<'a> {
        was_evaluated: &'a AtomicBool,
    }

    impl<'a> std::fmt::Display for EvalTracker<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.was_evaluated.store(true, Ordering::SeqCst);
            write!(f, "tracked")
        }
    }

    /// Validates: Requirement 3.5, Requirement 9.5
    ///
    /// When the logging subsystem is not initialized (SUBSYSTEM is None),
    /// `log_lazy` returns early before invoking the closure. This confirms
    /// that format arguments are never evaluated when the level would be
    /// filtered (or when there's no active subsystem).
    #[test]
    fn log_trace_macro_does_not_evaluate_args_when_subsystem_uninitialized() {
        let was_evaluated = AtomicBool::new(false);
        let tracker = EvalTracker {
            was_evaluated: &was_evaluated,
        };

        crate::log_trace!("value: {}", tracker);

        assert!(
            !was_evaluated.load(Ordering::SeqCst),
            "log_trace! should not evaluate format args when subsystem is not initialized"
        );
    }

    /// Validates: Requirement 3.5, Requirement 9.5
    #[test]
    fn log_debug_macro_does_not_evaluate_args_when_subsystem_uninitialized() {
        let was_evaluated = AtomicBool::new(false);
        let tracker = EvalTracker {
            was_evaluated: &was_evaluated,
        };

        crate::log_debug!("value: {}", tracker);

        assert!(
            !was_evaluated.load(Ordering::SeqCst),
            "log_debug! should not evaluate format args when subsystem is not initialized"
        );
    }

    /// Validates: Requirement 3.5, Requirement 9.5
    #[test]
    fn log_info_macro_does_not_evaluate_args_when_subsystem_uninitialized() {
        let was_evaluated = AtomicBool::new(false);
        let tracker = EvalTracker {
            was_evaluated: &was_evaluated,
        };

        crate::log_info!("value: {}", tracker);

        assert!(
            !was_evaluated.load(Ordering::SeqCst),
            "log_info! should not evaluate format args when subsystem is not initialized"
        );
    }

    /// Validates: Requirement 3.5, Requirement 9.5
    #[test]
    fn log_warn_macro_does_not_evaluate_args_when_subsystem_uninitialized() {
        let was_evaluated = AtomicBool::new(false);
        let tracker = EvalTracker {
            was_evaluated: &was_evaluated,
        };

        crate::log_warn!("value: {}", tracker);

        assert!(
            !was_evaluated.load(Ordering::SeqCst),
            "log_warn! should not evaluate format args when subsystem is not initialized"
        );
    }

    /// Validates: Requirement 3.5, Requirement 9.5
    #[test]
    fn log_error_macro_does_not_evaluate_args_when_subsystem_uninitialized() {
        let was_evaluated = AtomicBool::new(false);
        let tracker = EvalTracker {
            was_evaluated: &was_evaluated,
        };

        crate::log_error!("value: {}", tracker);

        assert!(
            !was_evaluated.load(Ordering::SeqCst),
            "log_error! should not evaluate format args when subsystem is not initialized"
        );
    }

    /// Validates: Requirement 3.5
    ///
    /// Verifies that the macros capture `module_path!()` automatically.
    /// Since we cannot easily inspect the module path in a unit test without
    /// a running subsystem, we verify that the macro compiles and expands
    /// correctly by calling it with various argument patterns.
    #[test]
    fn macros_compile_with_various_argument_patterns() {
        // No-arg format string
        crate::log_info!("simple message");

        // Single argument
        let x = 42;
        crate::log_debug!("value is {}", x);

        // Multiple arguments
        let name = "test";
        let count = 5;
        crate::log_trace!("{} has {} items", name, count);

        // Named arguments
        crate::log_warn!("item {name} count={count}");

        // Debug formatting
        let items = vec![1, 2, 3];
        crate::log_error!("items: {:?}", items);
    }
}
