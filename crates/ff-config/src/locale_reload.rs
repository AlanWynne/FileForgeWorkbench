//! Locale reload-callback wiring.
//!
//! Registers a hot-reload callback on the `ui.locale` key via the EXISTING
//! [`CallbackRegistry`], with NO new config mechanism introduced
//! (localization Requirement 1.4). When the effective value of `ui.locale`
//! changes at runtime, the registered action runs; the action is the seam the
//! workbench uses to reload BOTH the active Message_Catalogue AND the
//! per-locale, per-environment Alias_Catalogue (localization Requirement 1.5).
//!
//! `ff-config` deliberately does NOT own the catalogue-reload mechanism: the
//! real reload target (the `ff-i18n` catalogue swap and the Task 4 alias
//! loader) lives in the workbench, which supplies it as the `reload_action`
//! closure. This module only guarantees that a `ui.locale` change invokes that
//! action through the single existing callback path.

use crate::callback::{CallbackHandle, CallbackRegistry};
use crate::keys;

/// Register a reload action that fires whenever `ui.locale` changes.
///
/// The `reload_action` is the workbench-supplied seam that reloads the active
/// Message_Catalogue and the per-locale, per-environment Alias_Catalogue
/// (localization Requirement 1.5). It is invoked through the EXISTING
/// [`CallbackRegistry::on_reload`] mechanism keyed on
/// [`keys::ui::LOCALE`](crate::keys::ui::LOCALE), so no new config or reload
/// path is introduced (localization Requirement 1.4).
///
/// Returns the [`CallbackHandle`] so the caller can deregister the action
/// during shutdown, exactly as for any other reload callback.
///
/// # Examples
///
/// ```ignore
/// let handle = on_locale_reload(config.callbacks(), || {
///     // workbench: reload the ff-i18n catalogue + per-environment aliases
///     reload_message_catalogue(active_locale());
///     reload_alias_catalogue(active_locale());
/// });
/// ```
pub fn on_locale_reload<F>(callbacks: &CallbackRegistry, reload_action: F) -> CallbackHandle
where
    F: Fn() + Send + Sync + 'static,
{
    callbacks.on_reload(
        &[keys::ui::LOCALE],
        Box::new(move |_event| {
            reload_action();
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::ConfigLayer;
    use crate::reload::ReloadEvent;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::SystemTime;

    /// Helper: build a ReloadEvent for the given changed keys.
    fn event_for(keys: &[&str]) -> ReloadEvent {
        ReloadEvent {
            changed_keys: keys.iter().map(|k| (*k).to_string()).collect(),
            source_layer: ConfigLayer::User,
            timestamp: SystemTime::now(),
        }
    }

    // Validates: localization Requirement 1.4, 1.5 -- a ui.locale change fires
    // the registered reload action (the catalogue + alias reload seam) through
    // the existing CallbackRegistry, proven here with a spy counter.
    #[test]
    fn ui_locale_change_fires_reload_action() {
        let registry = CallbackRegistry::new();
        let fired = Arc::new(AtomicU32::new(0));
        let fired_clone = Arc::clone(&fired);

        let _handle = on_locale_reload(&registry, move || {
            // Stand-in for "reload Message_Catalogue AND Alias_Catalogue".
            fired_clone.fetch_add(1, Ordering::SeqCst);
        });

        registry.invoke(&event_for(&["ui.locale"]));

        assert_eq!(
            fired.load(Ordering::SeqCst),
            1,
            "ui.locale change must invoke the reload action exactly once"
        );
    }

    // Validates: localization Requirement 1.4 -- the action is scoped to
    // ui.locale and does NOT fire for an unrelated key change.
    #[test]
    fn unrelated_key_change_does_not_fire_locale_reload_action() {
        let registry = CallbackRegistry::new();
        let fired = Arc::new(AtomicU32::new(0));
        let fired_clone = Arc::clone(&fired);

        let _handle = on_locale_reload(&registry, move || {
            fired_clone.fetch_add(1, Ordering::SeqCst);
        });

        registry.invoke(&event_for(&["theme.active", "logging.level"]));

        assert_eq!(
            fired.load(Ordering::SeqCst),
            0,
            "a non-ui.locale change must not invoke the locale reload action"
        );
    }

    // Validates: localization Requirement 1.4 -- the returned handle deregisters
    // the action through the existing CallbackRegistry mechanism.
    #[test]
    fn locale_reload_action_can_be_deregistered() {
        let registry = CallbackRegistry::new();
        let fired = Arc::new(AtomicU32::new(0));
        let fired_clone = Arc::clone(&fired);

        let handle = on_locale_reload(&registry, move || {
            fired_clone.fetch_add(1, Ordering::SeqCst);
        });
        registry.remove_callback(handle);

        registry.invoke(&event_for(&["ui.locale"]));

        assert_eq!(
            fired.load(Ordering::SeqCst),
            0,
            "a deregistered locale reload action must not fire"
        );
    }
}
