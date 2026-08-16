//! Resource URI type for the VFS abstraction layer.
//!
//! Provides the [`ResourceUri`] type implementing the `vfs://provider/path` addressing scheme.
//! Every resource in the workbench is uniquely identified by a URI of this form.

use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use crate::VfsError;

/// A unified resource identifier in the format `vfs://provider/path`.
///
/// Uniquely identifies any resource regardless of its backing store.
/// The `provider` component selects the registered VFS provider, and the
/// `path` component is provider-specific.
///
/// # URI Format
///
/// ```text
/// vfs://provider/path?key=value&key2=value2
/// ```
///
/// - **provider**: alphanumeric, hyphen, or underscore characters (non-empty)
/// - **path**: provider-specific path (non-empty, starts with `/`)
/// - **query**: optional key-value parameters
///
/// # Examples
///
/// ```
/// use ff_vfs::ResourceUri;
///
/// let uri = ResourceUri::parse("vfs://local/home/user/file.txt").unwrap();
/// assert_eq!(uri.scheme(), "local");
/// assert_eq!(uri.path(), "/home/user/file.txt");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceUri {
    /// The provider scheme identifier (e.g., "local", "catalog").
    provider: String,
    /// The provider-specific path.
    path: String,
    /// Optional query parameters.
    query: Option<HashMap<String, String>>,
}

impl Hash for ResourceUri {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.provider.hash(state);
        self.path.hash(state);
        // Hash query parameters in sorted order for determinism
        if let Some(ref query) = self.query {
            let mut pairs: Vec<_> = query.iter().collect();
            pairs.sort_by_key(|(k, _)| k.as_str());
            for (key, value) in pairs {
                key.hash(state);
                value.hash(state);
            }
        }
    }
}

/// The URI scheme prefix that all VFS URIs must start with.
const VFS_SCHEME_PREFIX: &str = "vfs://";

impl ResourceUri {
    /// Parse a URI string into a `ResourceUri`.
    ///
    /// Validates that the scheme is `vfs`, the provider component is non-empty
    /// and contains only valid identifier characters (alphanumeric, hyphen,
    /// underscore), and the path component is non-empty.
    ///
    /// # Errors
    ///
    /// Returns [`VfsError::InvalidUri`] if validation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_vfs::ResourceUri;
    ///
    /// let uri = ResourceUri::parse("vfs://catalog/MY.DATASET.MEMBER").unwrap();
    /// assert_eq!(uri.scheme(), "catalog");
    /// assert_eq!(uri.path(), "/MY.DATASET.MEMBER");
    /// ```
    pub fn parse(uri: &str) -> Result<Self, VfsError> {
        // Check for the vfs:// prefix
        let remainder =
            uri.strip_prefix(VFS_SCHEME_PREFIX)
                .ok_or_else(|| VfsError::InvalidUri {
                    uri: uri.to_string(),
                    reason: "missing vfs:// scheme prefix".to_string(),
                })?;

        // Split provider from path at the first '/'
        let (provider, path_and_query) = match remainder.find('/') {
            Some(idx) => (&remainder[..idx], &remainder[idx..]),
            None => {
                // No path separator — the entire remainder is the provider with no path
                return Err(VfsError::InvalidUri {
                    uri: uri.to_string(),
                    reason: "missing path component".to_string(),
                });
            }
        };

        // Validate provider is non-empty
        if provider.is_empty() {
            return Err(VfsError::InvalidUri {
                uri: uri.to_string(),
                reason: "provider component is empty".to_string(),
            });
        }

        // Validate provider characters: alphanumeric, hyphen, underscore only
        if !provider
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(VfsError::InvalidUri {
                uri: uri.to_string(),
                reason: format!(
                    "provider '{}' contains invalid characters (only alphanumeric, hyphen, underscore allowed)",
                    provider
                ),
            });
        }

        // Split path from query parameters
        let (path, query) = match path_and_query.find('?') {
            Some(idx) => (&path_and_query[..idx], Some(&path_and_query[idx + 1..])),
            None => (path_and_query, None),
        };

        // Validate path is non-empty (must have content beyond just '/')
        if path.is_empty() || path == "/" {
            return Err(VfsError::InvalidUri {
                uri: uri.to_string(),
                reason: "path component is empty".to_string(),
            });
        }

        // Parse query parameters if present
        let query_map = query.map(|q| {
            q.split('&')
                .filter(|pair| !pair.is_empty())
                .filter_map(|pair| {
                    let mut parts = pair.splitn(2, '=');
                    let key = parts.next()?;
                    let value = parts.next().unwrap_or("");
                    if key.is_empty() {
                        None
                    } else {
                        Some((key.to_string(), value.to_string()))
                    }
                })
                .collect::<HashMap<String, String>>()
        });

        // If the query map is empty, treat it as None
        let query_map = query_map.filter(|m| !m.is_empty());

        Ok(Self {
            provider: provider.to_string(),
            path: path.to_string(),
            query: query_map,
        })
    }

    /// Construct a `ResourceUri` from components without re-parsing.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_vfs::ResourceUri;
    ///
    /// let uri = ResourceUri::new("local", "/home/user/file.txt");
    /// assert_eq!(uri.scheme(), "local");
    /// assert_eq!(uri.path(), "/home/user/file.txt");
    /// ```
    pub fn new(provider: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            path: path.into(),
            query: None,
        }
    }

    /// Construct a `ResourceUri` with query parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use ff_vfs::ResourceUri;
    ///
    /// let mut query = HashMap::new();
    /// query.insert("encoding".to_string(), "utf-8".to_string());
    /// let uri = ResourceUri::with_query("local", "/file.txt", query);
    /// assert_eq!(uri.query().unwrap().get("encoding").unwrap(), "utf-8");
    /// ```
    pub fn with_query(
        provider: impl Into<String>,
        path: impl Into<String>,
        query: HashMap<String, String>,
    ) -> Self {
        Self {
            provider: provider.into(),
            path: path.into(),
            query: if query.is_empty() { None } else { Some(query) },
        }
    }

    /// Interpret a bare path as a local filesystem URI.
    ///
    /// The provider defaults to `"local"`. The path is used as-is if it
    /// starts with `/`; otherwise a leading `/` is prepended.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_vfs::ResourceUri;
    ///
    /// let uri = ResourceUri::from_path("/home/user/file.txt");
    /// assert_eq!(uri.scheme(), "local");
    /// assert_eq!(uri.path(), "/home/user/file.txt");
    /// ```
    pub fn from_path(path: impl Into<String>) -> Self {
        let path_str = path.into();
        let normalized = if path_str.starts_with('/') {
            path_str
        } else {
            format!("/{}", path_str)
        };
        Self {
            provider: "local".to_string(),
            path: normalized,
            query: None,
        }
    }

    /// Get the provider scheme identifier.
    pub fn scheme(&self) -> &str {
        &self.provider
    }

    /// Get the provider-specific path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get optional query parameters.
    pub fn query(&self) -> Option<&HashMap<String, String>> {
        self.query.as_ref()
    }

    /// Return the canonical string representation of this URI.
    ///
    /// Equivalent to calling `.to_string()` via the `Display` implementation.
    pub fn as_str(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for ResourceUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vfs://{}{}", self.provider, self.path)?;
        if let Some(ref query) = self.query {
            let mut pairs: Vec<_> = query.iter().collect();
            // Sort for deterministic output
            pairs.sort_by_key(|(k, _)| k.as_str());
            write!(f, "?")?;
            for (i, (key, value)) in pairs.iter().enumerate() {
                if i > 0 {
                    write!(f, "&")?;
                }
                write!(f, "{}={}", key, value)?;
            }
        }
        Ok(())
    }
}

impl FromStr for ResourceUri {
    type Err = VfsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2 AC 1, AC 3
    #[test]
    fn parse_valid_uri_extracts_provider_and_path() {
        let uri = ResourceUri::parse("vfs://local/home/user/file.txt").unwrap();
        assert_eq!(uri.scheme(), "local");
        assert_eq!(uri.path(), "/home/user/file.txt");
        assert_eq!(uri.query(), None);
    }

    // Validates: Requirement 2 AC 1, AC 6
    #[test]
    fn parse_valid_uri_with_hyphen_underscore_provider() {
        let uri = ResourceUri::parse("vfs://my-provider_1/data/resource").unwrap();
        assert_eq!(uri.scheme(), "my-provider_1");
        assert_eq!(uri.path(), "/data/resource");
    }

    // Validates: Requirement 2 AC 8
    #[test]
    fn parse_valid_uri_with_query_parameters() {
        let uri = ResourceUri::parse("vfs://catalog/MY.DATASET?encoding=ebcdic&recfm=fb").unwrap();
        assert_eq!(uri.scheme(), "catalog");
        assert_eq!(uri.path(), "/MY.DATASET");
        let query = uri.query().unwrap();
        assert_eq!(query.get("encoding").unwrap(), "ebcdic");
        assert_eq!(query.get("recfm").unwrap(), "fb");
    }

    // Validates: Requirement 2 AC 4, AC 5
    #[test]
    fn parse_rejects_missing_vfs_prefix() {
        let err = ResourceUri::parse("http://local/file.txt").unwrap_err();
        match err {
            VfsError::InvalidUri { uri, reason } => {
                assert_eq!(uri, "http://local/file.txt");
                assert!(reason.contains("missing vfs:// scheme prefix"));
            }
            _ => panic!("expected InvalidUri error"),
        }
    }

    // Validates: Requirement 2 AC 4, AC 5
    #[test]
    fn parse_rejects_empty_provider() {
        let err = ResourceUri::parse("vfs:///some/path").unwrap_err();
        match err {
            VfsError::InvalidUri { uri, reason } => {
                assert_eq!(uri, "vfs:///some/path");
                assert!(reason.contains("provider component is empty"));
            }
            _ => panic!("expected InvalidUri error"),
        }
    }

    // Validates: Requirement 2 AC 4, AC 5
    #[test]
    fn parse_rejects_invalid_provider_characters() {
        let err = ResourceUri::parse("vfs://bad provider/path").unwrap_err();
        match err {
            VfsError::InvalidUri { uri, reason } => {
                assert_eq!(uri, "vfs://bad provider/path");
                assert!(reason.contains("invalid characters"));
            }
            _ => panic!("expected InvalidUri error"),
        }
    }

    // Validates: Requirement 2 AC 4, AC 5
    #[test]
    fn parse_rejects_empty_path() {
        let err = ResourceUri::parse("vfs://local/").unwrap_err();
        match err {
            VfsError::InvalidUri { uri, reason } => {
                assert_eq!(uri, "vfs://local/");
                assert!(reason.contains("path component is empty"));
            }
            _ => panic!("expected InvalidUri error"),
        }
    }

    // Validates: Requirement 2 AC 4, AC 5
    #[test]
    fn parse_rejects_no_path_separator() {
        let err = ResourceUri::parse("vfs://local").unwrap_err();
        match err {
            VfsError::InvalidUri { uri, reason } => {
                assert_eq!(uri, "vfs://local");
                assert!(reason.contains("missing path component"));
            }
            _ => panic!("expected InvalidUri error"),
        }
    }

    // Validates: Requirement 2 AC 3
    #[test]
    fn new_constructor_creates_uri_without_parsing() {
        let uri = ResourceUri::new("catalog", "/MY.DATASET");
        assert_eq!(uri.scheme(), "catalog");
        assert_eq!(uri.path(), "/MY.DATASET");
        assert_eq!(uri.query(), None);
    }

    // Validates: Requirement 2 AC 8
    #[test]
    fn with_query_constructor_stores_parameters() {
        let mut query = HashMap::new();
        query.insert("key".to_string(), "value".to_string());
        let uri = ResourceUri::with_query("local", "/file.txt", query);
        assert_eq!(uri.query().unwrap().get("key").unwrap(), "value");
    }

    // Validates: Requirement 2 AC 8
    #[test]
    fn with_query_empty_map_results_in_none() {
        let uri = ResourceUri::with_query("local", "/file.txt", HashMap::new());
        assert_eq!(uri.query(), None);
    }

    // Validates: Requirement 2 AC 10
    #[test]
    fn from_path_defaults_to_local_provider() {
        let uri = ResourceUri::from_path("/home/user/file.txt");
        assert_eq!(uri.scheme(), "local");
        assert_eq!(uri.path(), "/home/user/file.txt");
    }

    // Validates: Requirement 2 AC 10
    #[test]
    fn from_path_prepends_slash_if_missing() {
        let uri = ResourceUri::from_path("relative/path.txt");
        assert_eq!(uri.scheme(), "local");
        assert_eq!(uri.path(), "/relative/path.txt");
    }

    // Validates: Requirement 2 AC 9
    #[test]
    fn display_produces_canonical_uri_string() {
        let uri = ResourceUri::new("local", "/home/user/file.txt");
        assert_eq!(uri.to_string(), "vfs://local/home/user/file.txt");
    }

    // Validates: Requirement 2 AC 9
    #[test]
    fn display_includes_sorted_query_parameters() {
        let mut query = HashMap::new();
        query.insert("b".to_string(), "2".to_string());
        query.insert("a".to_string(), "1".to_string());
        let uri = ResourceUri::with_query("local", "/file.txt", query);
        assert_eq!(uri.to_string(), "vfs://local/file.txt?a=1&b=2");
    }

    // Validates: Requirement 2 AC 9
    #[test]
    fn from_str_delegates_to_parse() {
        let uri: ResourceUri = "vfs://local/home/user/file.txt".parse().unwrap();
        assert_eq!(uri.scheme(), "local");
        assert_eq!(uri.path(), "/home/user/file.txt");
    }

    // Validates: Requirement 2 AC 9
    #[test]
    fn from_str_rejects_invalid_input() {
        let result: Result<ResourceUri, _> = "not-a-uri".parse();
        assert!(result.is_err());
    }

    // Validates: Requirement 2 AC 3
    #[test]
    fn as_str_returns_canonical_string() {
        let uri = ResourceUri::new("local", "/file.txt");
        assert_eq!(uri.as_str(), "vfs://local/file.txt");
    }

    // Validates: Requirement 2 AC 9
    #[test]
    fn clone_and_eq_work_correctly() {
        let uri = ResourceUri::new("local", "/file.txt");
        let cloned = uri.clone();
        assert_eq!(uri, cloned);
    }

    // Validates: Requirement 2 AC 9
    #[test]
    fn hash_is_consistent_for_equal_uris() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let uri1 = ResourceUri::new("local", "/file.txt");
        let uri2 = ResourceUri::new("local", "/file.txt");

        let mut hasher1 = DefaultHasher::new();
        uri1.hash(&mut hasher1);
        let mut hasher2 = DefaultHasher::new();
        uri2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    // Validates: Requirement 2 AC 4
    #[test]
    fn parse_rejects_provider_with_special_characters() {
        let err = ResourceUri::parse("vfs://bad.provider/path").unwrap_err();
        match err {
            VfsError::InvalidUri { reason, .. } => {
                assert!(reason.contains("invalid characters"));
            }
            _ => panic!("expected InvalidUri error"),
        }
    }
}
