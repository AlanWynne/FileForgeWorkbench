//! The Catalogue_Lookup_Seam: free `t` / `t_args` functions over a shared,
//! swappable active [`Catalogue`].
//!
//! The active catalogue lives behind a process-wide `RwLock` so a render site
//! calls the free functions `t("key")` / `t_args("key", args)` without threading
//! a catalogue through every call (localization Req 3.4, Task 1.5). A hot-reload
//! swaps the catalogue via [`set_catalogue`] without touching any call site.
//!
//! Before a catalogue is installed, the seam returns the key itself (the same
//! visible fallback as a missing key) so lookups never panic and never render
//! blank text (localization Req 3.3).

use std::sync::{OnceLock, RwLock};

use crate::args::MessageArgs;
use crate::catalogue::Catalogue;

/// The process-wide active catalogue. `None` until [`set_catalogue`] installs
/// one; the seam degrades gracefully to the key fallback until then.
fn active() -> &'static RwLock<Option<Catalogue>> {
    static ACTIVE: OnceLock<RwLock<Option<Catalogue>>> = OnceLock::new();
    ACTIVE.get_or_init(|| RwLock::new(None))
}

/// Install (or replace) the active catalogue.
///
/// Called once at startup and again on each `ui.locale` hot-reload to swap the
/// resolved catalogue atomically for all subsequent lookups (localization Req
/// 3.4, Task 1.5). The swap is behind the shared lock, so in-flight lookups
/// either see the old or the new catalogue, never a partial state.
pub fn set_catalogue(catalogue: Catalogue) {
    match active().write() {
        Ok(mut guard) => *guard = Some(catalogue),
        Err(poisoned) => {
            // A poisoned lock means a previous lookup panicked while holding the
            // guard. Recover by replacing the inner value so the seam keeps
            // working rather than propagating the panic.
            *poisoned.into_inner() = Some(catalogue);
        }
    }
}

/// Whether an active catalogue is currently installed.
pub fn is_initialised() -> bool {
    active()
        .read()
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

/// Resolve a message `key` against the active catalogue.
///
/// Returns the active-locale string, falling back to the English Identity_Base,
/// then to the key itself with a WARN when it is missing everywhere
/// (localization Req 3.2, 3.3). The owned `String` is what egui widgets already
/// accept, so a render site swaps a literal for this call with no API change
/// (localization Req 3.4).
pub fn t(key: &str) -> String {
    t_args(key, &MessageArgs::new())
}

/// Resolve a message `key` supplying named placeable arguments.
///
/// Same fallback chain as [`t`]. Use this for interpolated runtime values and
/// for an embedded command verb carried as a non-translated argument
/// (localization Req 9.1, 9.2).
pub fn t_args(key: &str, args: &MessageArgs) -> String {
    let guard = match active().read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    match guard.as_ref() {
        Some(catalogue) => catalogue.resolve_args(key, args),
        None => {
            ff_logging::log_warn!(
                "[i18n] lookup for key \"{}\" before a catalogue was installed -- returning the key as fallback",
                key
            );
            key.to_string()
        }
    }
}
