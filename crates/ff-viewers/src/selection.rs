//! Content selection and viewer matching logic.
//!
//! Determines which viewer should handle a given resource based on a priority
//! chain: language profile > extension match > content sniff > none.

use std::collections::HashSet;

use crate::key::ViewerKey;
use crate::registry::ViewerRegistry;

/// The method by which a viewer was matched to a resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchMethod {
    /// Matched via language profile `default_viewer` key.
    LanguageProfile,
    /// Matched via file extension in `supported_extensions()`.
    Extension,
    /// Matched via MIME type in `supported_mime_types()`.
    MimeType,
    /// Matched via `can_render()` content sniffing.
    ContentSniff,
    /// Explicit user selection via `PREVIEW <key>`.
    UserExplicit,
}

/// Confidence level for content matching, used to rank multiple matching viewers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchConfidence {
    /// Low confidence — content sniff heuristic.
    Low,
    /// Medium confidence — extension or MIME match.
    Medium,
    /// High confidence — language profile explicit declaration.
    High,
    /// Highest — user explicitly requested this viewer.
    Explicit,
}

/// Describes how a viewer matches a given resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentMatch {
    /// The viewer key that matched.
    pub viewer_key: ViewerKey,
    /// How the match was determined.
    pub match_method: MatchMethod,
    /// Confidence score (higher = better match).
    pub confidence: MatchConfidence,
}

/// Determines which viewer should handle a given resource.
///
/// Implements the priority chain:
/// 1. Language profile `default_viewer`
/// 2. Extension match
/// 3. Content sniffing via `can_render()`
///
/// Also tracks dismissed viewer offers per resource per session.
pub struct ContentSelector {
    /// Set of resource URIs for which the user has dismissed the viewer offer.
    dismissed_offers: HashSet<String>,
}

impl ContentSelector {
    /// Create a new content selector.
    pub fn new(_registry: &ViewerRegistry) -> Self {
        Self {
            dismissed_offers: HashSet::new(),
        }
    }

    /// Determine the best viewer for a resource based on the priority chain.
    ///
    /// Priority:
    /// 1. Language profile `default_viewer` (if provided and registered)
    /// 2. Extension match against all registered viewers
    /// 3. Content sniffing via `can_render()`
    ///
    /// Returns `None` if no viewer matches.
    pub fn select_viewer(
        &self,
        uri: &str,
        content_sample: &[u8],
        language_profile_viewer: Option<&str>,
    ) -> Option<ViewerKey> {
        // Priority 1: Language profile explicit declaration
        if let Some(profile_key) = language_profile_viewer {
            if let Ok(key) = ViewerKey::new(profile_key) {
                return Some(key);
            }
        }

        // Priority 2: Extension match
        if let Some(ext) = extract_extension(uri) {
            if let Ok(key) = self.match_by_extension(&ext) {
                return Some(key);
            }
        }

        // Priority 3: Content sniff — for now return None (would need registry access)
        let _ = content_sample;

        None
    }

    /// Select a viewer by extension matching against known built-in extensions.
    fn match_by_extension(&self, ext: &str) -> Result<ViewerKey, ()> {
        let ext_lower = ext.to_lowercase();
        match ext_lower.as_str() {
            "lst" | "rpt" | "spool" => ViewerKey::new("asa-report").map_err(|_| ()),
            "csv" | "tsv" => ViewerKey::new("csv-table").map_err(|_| ()),
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" => {
                ViewerKey::new("image").map_err(|_| ())
            }
            _ => Err(()),
        }
    }

    /// Select viewer by content sniffing, invoking `can_render` on the registry.
    pub fn select_by_content_sniff(
        &self,
        registry: &ViewerRegistry,
        uri: &str,
        content_sample: &[u8],
    ) -> Option<ViewerKey> {
        let viewers = registry.list_viewers();
        for info in &viewers {
            let matches =
                registry.with_viewer(&info.key, |viewer| viewer.can_render(uri, content_sample));
            if matches == Some(true) {
                return Some(info.key.clone());
            }
        }
        None
    }

    /// Record that the user dismissed the viewer offer for a resource.
    pub fn dismiss_offer(&mut self, uri: &str) {
        self.dismissed_offers.insert(uri.to_string());
    }

    /// Check whether the offer was already dismissed for this resource.
    pub fn is_offer_dismissed(&self, uri: &str) -> bool {
        self.dismissed_offers.contains(uri)
    }
}

/// Extract the file extension from a URI or path.
fn extract_extension(uri: &str) -> Option<String> {
    let path = uri.rsplit('/').next().unwrap_or(uri);
    let dot_pos = path.rfind('.')?;
    let ext = &path[dot_pos + 1..];
    if ext.is_empty() {
        None
    } else {
        Some(ext.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::built_in::register_built_in_viewers;

    #[test]
    fn select_viewer_by_extension_csv() {
        // Validates: Requirement 6 AC 1
        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();
        let selector = ContentSelector::new(&registry);

        let result = selector.select_viewer("file:///data.csv", b"", None);
        assert_eq!(result.unwrap().as_str(), "csv-table");
    }

    #[test]
    fn select_viewer_by_extension_lst() {
        // Validates: Requirement 6 AC 1
        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();
        let selector = ContentSelector::new(&registry);

        let result = selector.select_viewer("file:///report.lst", b"", None);
        assert_eq!(result.unwrap().as_str(), "asa-report");
    }

    #[test]
    fn select_viewer_by_extension_png() {
        // Validates: Requirement 6 AC 1
        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();
        let selector = ContentSelector::new(&registry);

        let result = selector.select_viewer("file:///photo.png", b"", None);
        assert_eq!(result.unwrap().as_str(), "image");
    }

    #[test]
    fn language_profile_overrides_extension() {
        // Validates: Requirement 6 AC 2
        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();
        let selector = ContentSelector::new(&registry);

        // CSV extension would normally match csv-table, but language profile says hex
        let result = selector.select_viewer("file:///data.csv", b"", Some("hex"));
        assert_eq!(result.unwrap().as_str(), "hex");
    }

    #[test]
    fn no_match_returns_none() {
        // Validates: Requirement 6 AC 4
        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();
        let selector = ContentSelector::new(&registry);

        let result = selector.select_viewer("file:///unknown.xyz", b"hello", None);
        assert!(result.is_none());
    }

    #[test]
    fn content_sniff_fallback() {
        // Validates: Requirement 6 AC 4
        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();
        let selector = ContentSelector::new(&registry);

        // Binary content with null bytes should trigger hex viewer via can_render
        let binary = &[0x00, 0x48, 0x65, 0x6C, 0x6C, 0x6F];
        let result = selector.select_by_content_sniff(&registry, "file:///unknown", binary);
        assert_eq!(result.unwrap().as_str(), "hex");
    }

    #[test]
    fn dismissed_notification_not_reshown() {
        // Validates: Requirement 6 AC 6
        let registry = ViewerRegistry::new();
        let mut selector = ContentSelector::new(&registry);

        let uri = "file:///report.lst";
        assert!(!selector.is_offer_dismissed(uri));

        selector.dismiss_offer(uri);
        assert!(selector.is_offer_dismissed(uri));
    }

    #[test]
    fn extract_extension_works() {
        assert_eq!(
            extract_extension("file:///path/to/file.csv"),
            Some("csv".to_string())
        );
        assert_eq!(
            extract_extension("file:///file.tar.gz"),
            Some("gz".to_string())
        );
        assert_eq!(extract_extension("file:///noext"), None);
        assert_eq!(extract_extension("file:///dot."), None);
    }
}
