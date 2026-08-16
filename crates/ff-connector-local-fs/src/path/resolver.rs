//! PathResolver — tilde expansion, environment variable substitution,
//! relative path resolution, canonicalization, and URI ↔ native conversion.
//!
//! Addresses: Requirement 2 (cross-platform), Requirement 4 (path resolution)

use std::env;
use std::path::{Component, Path, PathBuf};

use ff_vfs::VfsError;

use super::NativePath;

/// Handles all path resolution: tilde expansion, environment variables,
/// relative path resolution, canonicalization, and URI ↔ native conversion.
///
/// Addresses: Requirement 2 (cross-platform), Requirement 4 (path resolution)
pub struct PathResolver {
    /// The current working directory (captured at construction).
    working_directory: PathBuf,
    /// The user's home directory (captured at construction).
    home_directory: PathBuf,
}

impl PathResolver {
    /// Construct a new `PathResolver`, capturing the current working directory
    /// and home directory from the environment.
    ///
    /// Validates: Requirement 4 AC 1, AC 2
    pub fn new() -> Result<Self, VfsError> {
        let working_directory = env::current_dir().map_err(|e| VfsError::Io {
            uri: String::new(),
            operation: "init_path_resolver".to_string(),
            source: e,
        })?;

        let home_directory = dirs::home_dir().ok_or_else(|| VfsError::Io {
            uri: String::new(),
            operation: "init_path_resolver".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "home directory not found"),
        })?;

        Ok(Self {
            working_directory,
            home_directory,
        })
    }

    /// Construct with explicit working directory and home directory (for testing).
    pub fn with_dirs(working_dir: PathBuf, home_dir: PathBuf) -> Self {
        Self {
            working_directory: working_dir,
            home_directory: home_dir,
        }
    }

    /// Returns a reference to the home directory.
    pub fn home_directory(&self) -> &Path {
        &self.home_directory
    }

    /// Returns a reference to the working directory.
    pub fn working_directory(&self) -> &Path {
        &self.working_directory
    }

    /// Resolve a VFS path string to a `NativePath`.
    ///
    /// Handles: relative paths, tilde expansion, env var expansion, `.`/`..` segments.
    ///
    /// Validates: Requirement 4, criteria 1–6
    pub fn resolve(&self, path: &str) -> Result<NativePath, VfsError> {
        // Step 1: Expand tilde
        let expanded_tilde = self.expand_tilde(path);

        // Step 2: Expand environment variables
        let expanded_env = self.expand_env_vars(&expanded_tilde)?;

        // Step 3: Convert to PathBuf
        let path_buf = PathBuf::from(&expanded_env);

        // Step 4: Resolve relative paths against working directory
        let absolute = if path_buf.is_absolute() {
            path_buf
        } else {
            self.working_directory.join(&path_buf)
        };

        // Step 5: Normalize `.` and `..` segments (logical, no filesystem access)
        let normalized = normalise_path(&absolute);

        Ok(NativePath::from_path_buf(normalized))
    }

    /// Canonicalize a path: resolve all symlinks, eliminate all `.`/`..`,
    /// produce the true absolute path as reported by the OS.
    ///
    /// Validates: Requirement 4, criteria 7–8
    pub async fn canonicalize(&self, path: &str) -> Result<NativePath, VfsError> {
        let resolved = self.resolve(path)?;
        let canonical = tokio::fs::canonicalize(resolved.as_path())
            .await
            .map_err(|e| {
                crate::error::map_io_error(e, "canonicalize", &format!("vfs://local{}", path))
            })?;
        Ok(NativePath::from_path_buf(canonical))
    }

    /// Expand tilde prefix to home directory.
    ///
    /// Replaces `~/` or `~\` at the start of a path with the user's home directory.
    ///
    /// Validates: Requirement 4, criterion 2
    pub fn expand_tilde(&self, path: &str) -> String {
        if path == "~" {
            return self.home_directory.to_string_lossy().to_string();
        }
        if path.starts_with("~/") || path.starts_with("~\\") {
            let home = self.home_directory.to_string_lossy();
            return format!("{}{}", home, &path[1..]);
        }
        path.to_string()
    }

    /// Expand environment variables in a path string.
    ///
    /// Supports both Unix (`$VAR`, `${VAR}`) and Windows (`%VAR%`) syntax.
    /// Returns `VfsError::InvalidUri` (as a proxy for InvalidPath) on undefined variables.
    ///
    /// Validates: Requirement 4, criteria 3–5
    pub fn expand_env_vars(&self, path: &str) -> Result<String, VfsError> {
        let mut result = String::with_capacity(path.len());
        let mut chars = path.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '$' => {
                    // Unix-style variable: ${VAR} or $VAR
                    let var_name = if chars.peek() == Some(&'{') {
                        chars.next(); // consume '{'
                        let name: String = chars.by_ref().take_while(|&ch| ch != '}').collect();
                        name
                    } else {
                        // Collect alphanumeric/underscore chars without consuming the delimiter
                        let mut name = String::new();
                        while let Some(&ch) = chars.peek() {
                            if ch.is_alphanumeric() || ch == '_' {
                                name.push(ch);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        name
                    };

                    if var_name.is_empty() {
                        result.push('$');
                        continue;
                    }

                    match env::var(&var_name) {
                        Ok(value) => result.push_str(&value),
                        Err(_) => {
                            return Err(VfsError::InvalidUri {
                                uri: path.to_string(),
                                reason: format!("undefined environment variable: {}", var_name),
                            });
                        }
                    }
                }
                '%' => {
                    // Windows-style variable: %VAR%
                    let name: String = chars.by_ref().take_while(|&ch| ch != '%').collect();
                    if name.is_empty() {
                        result.push('%');
                        continue;
                    }

                    match env::var(&name) {
                        Ok(value) => result.push_str(&value),
                        Err(_) => {
                            return Err(VfsError::InvalidUri {
                                uri: path.to_string(),
                                reason: format!("undefined environment variable: {}", name),
                            });
                        }
                    }
                }
                _ => result.push(c),
            }
        }

        Ok(result)
    }

    /// Normalise path separators to the platform-native separator.
    ///
    /// On Windows, converts `/` to `\`. On Unix, converts `\` to `/`.
    ///
    /// Validates: Requirement 2 AC 3
    pub fn normalise_separators(path: &str) -> String {
        #[cfg(windows)]
        {
            path.replace('/', "\\")
        }
        #[cfg(not(windows))]
        {
            path.replace('\\', "/")
        }
    }

    /// Compare two paths for equality using platform-appropriate rules.
    ///
    /// Case-insensitive on Windows, case-sensitive on Unix.
    ///
    /// Validates: Requirement 2, criteria 4–5
    pub fn paths_equal(a: &NativePath, b: &NativePath) -> bool {
        #[cfg(windows)]
        {
            a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
        }
        #[cfg(not(windows))]
        {
            a.as_path() == b.as_path()
        }
    }

    /// Convert a `NativePath` to a VFS URI path component.
    ///
    /// Produces the path portion of `vfs://local/...`.
    ///
    /// Validates: Requirement 4, criterion 9; Requirement 2, criterion 10
    pub fn native_to_uri_path(native: &NativePath) -> String {
        let path_str = native.to_string_lossy();

        // Normalise separators to forward slashes for URI
        let normalized = path_str.replace('\\', "/");

        // On Windows, handle drive letters: C:\path → /C:/path
        #[cfg(windows)]
        {
            if normalized.len() >= 2 && normalized.as_bytes()[1] == b':' {
                return format!("/{}", normalized);
            }
        }

        // Ensure leading slash
        if normalized.starts_with('/') {
            normalized
        } else {
            format!("/{}", normalized)
        }
    }

    /// Convert a VFS URI path component to a `NativePath`.
    ///
    /// Validates: Requirement 2, criterion 9
    pub fn uri_path_to_native(uri_path: &str) -> Result<NativePath, VfsError> {
        let decoded = percent_decode(uri_path);

        #[cfg(windows)]
        {
            // Handle /C:/path → C:\path
            if decoded.len() >= 3 && decoded.as_bytes()[0] == b'/' && decoded.as_bytes()[2] == b':'
            {
                let without_leading_slash = &decoded[1..];
                let native = without_leading_slash.replace('/', "\\");
                return Ok(NativePath::from_path_buf(PathBuf::from(native)));
            }
        }

        // Unix paths stay as-is
        let native_str = Self::normalise_separators(&decoded);
        Ok(NativePath::from_path_buf(PathBuf::from(native_str)))
    }
}

/// Normalise a path by resolving `.` and `..` segments logically (no filesystem access).
///
/// Validates: Requirement 4 AC 6
fn normalise_path(path: &Path) -> PathBuf {
    let mut components: Vec<Component> = Vec::new();

    for component in path.components() {
        match component {
            Component::CurDir => {
                // Skip `.` segments
            }
            Component::ParentDir => {
                // Pop the last normal component if possible
                match components.last() {
                    Some(Component::Normal(_)) => {
                        components.pop();
                    }
                    _ => {
                        components.push(component);
                    }
                }
            }
            _ => {
                components.push(component);
            }
        }
    }

    components.iter().collect()
}

/// Simple percent-decoding for URI paths.
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                result.push(hi << 4 | lo);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i]);
        i += 1;
    }

    String::from_utf8_lossy(&result).to_string()
}

/// Convert a hex character to its numeric value.
fn hex_val(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_resolver() -> PathResolver {
        PathResolver::with_dirs(
            PathBuf::from("/workspace/project"),
            PathBuf::from("/home/testuser"),
        )
    }

    // Validates: Requirement 4 AC 2
    #[test]
    fn expand_tilde_replaces_tilde_with_home_dir() {
        let resolver = test_resolver();
        let result = resolver.expand_tilde("~/documents/file.txt");
        assert_eq!(result, "/home/testuser/documents/file.txt");
    }

    // Validates: Requirement 4 AC 2
    #[test]
    fn expand_tilde_lone_tilde_returns_home_dir() {
        let resolver = test_resolver();
        let result = resolver.expand_tilde("~");
        assert_eq!(result, "/home/testuser");
    }

    // Validates: Requirement 4 AC 2
    #[test]
    fn expand_tilde_no_tilde_returns_unchanged() {
        let resolver = test_resolver();
        let result = resolver.expand_tilde("/absolute/path");
        assert_eq!(result, "/absolute/path");
    }

    // Validates: Requirement 4 AC 3
    #[test]
    fn expand_env_vars_expands_dollar_syntax() {
        let resolver = test_resolver();
        env::set_var("TEST_CONNECTOR_VAR", "/tmp/test");
        let result = resolver
            .expand_env_vars("$TEST_CONNECTOR_VAR/file.txt")
            .unwrap();
        assert_eq!(result, "/tmp/test/file.txt");
        env::remove_var("TEST_CONNECTOR_VAR");
    }

    // Validates: Requirement 4 AC 3
    #[test]
    fn expand_env_vars_expands_braced_syntax() {
        let resolver = test_resolver();
        env::set_var("TEST_CONNECTOR_BRACED", "/opt/data");
        let result = resolver
            .expand_env_vars("${TEST_CONNECTOR_BRACED}/file.txt")
            .unwrap();
        assert_eq!(result, "/opt/data/file.txt");
        env::remove_var("TEST_CONNECTOR_BRACED");
    }

    // Validates: Requirement 4 AC 4
    #[test]
    fn expand_env_vars_expands_percent_syntax() {
        let resolver = test_resolver();
        env::set_var("TEST_CONNECTOR_WIN", "C:\\Users\\test");
        let result = resolver
            .expand_env_vars("%TEST_CONNECTOR_WIN%\\file.txt")
            .unwrap();
        assert_eq!(result, "C:\\Users\\test\\file.txt");
        env::remove_var("TEST_CONNECTOR_WIN");
    }

    // Validates: Requirement 4 AC 5
    #[test]
    fn expand_env_vars_undefined_variable_returns_error() {
        let resolver = test_resolver();
        let result = resolver.expand_env_vars("$UNDEFINED_VAR_XYZ_CONNECTOR/file.txt");
        assert!(result.is_err());
    }

    // Validates: Requirement 4 AC 1
    #[test]
    fn resolve_relative_path_prepends_working_dir() {
        let resolver = test_resolver();
        let result = resolver.resolve("src/main.rs").unwrap();
        assert_eq!(
            result.as_path(),
            Path::new("/workspace/project/src/main.rs")
        );
    }

    // Validates: Requirement 4 AC 6
    #[test]
    fn resolve_eliminates_dot_segments() {
        let resolver = test_resolver();
        let result = resolver
            .resolve("/home/user/./documents/../file.txt")
            .unwrap();
        assert_eq!(result.as_path(), Path::new("/home/user/file.txt"));
    }

    // Validates: Requirement 4 AC 6
    #[test]
    fn resolve_eliminates_dotdot_segments() {
        let resolver = test_resolver();
        let result = resolver.resolve("/a/b/c/../../d").unwrap();
        assert_eq!(result.as_path(), Path::new("/a/d"));
    }

    // Validates: Requirement 2 AC 3
    #[test]
    fn normalise_separators_converts_to_platform_native() {
        #[cfg(not(windows))]
        {
            let result = PathResolver::normalise_separators("a\\b\\c");
            assert_eq!(result, "a/b/c");
        }
        #[cfg(windows)]
        {
            let result = PathResolver::normalise_separators("a/b/c");
            assert_eq!(result, "a\\b\\c");
        }
    }

    // Validates: Requirement 2 AC 4, AC 5
    #[test]
    fn paths_equal_case_sensitivity() {
        let a = NativePath::new_from("/home/User/File.txt");
        let b = NativePath::new_from("/home/user/file.txt");

        #[cfg(windows)]
        assert!(PathResolver::paths_equal(&a, &b));

        #[cfg(not(windows))]
        assert!(!PathResolver::paths_equal(&a, &b));
    }

    // Validates: Requirement 4 AC 9, Requirement 2 AC 8
    #[test]
    fn native_to_uri_path_produces_slash_separated() {
        let native = NativePath::new_from("/home/user/file.txt");
        let uri = PathResolver::native_to_uri_path(&native);
        assert_eq!(uri, "/home/user/file.txt");
    }

    // Validates: Requirement 2 AC 9
    #[test]
    fn uri_path_to_native_unix_path() {
        let native = PathResolver::uri_path_to_native("/home/user/file.txt").unwrap();
        assert_eq!(native.as_path(), Path::new("/home/user/file.txt"));
    }

    #[cfg(windows)]
    #[test]
    fn native_to_uri_path_windows_drive_letter() {
        let native = NativePath::new_from(r"C:\Users\test\file.txt");
        let uri = PathResolver::native_to_uri_path(&native);
        assert_eq!(uri, "/C:/Users/test/file.txt");
    }

    #[cfg(windows)]
    #[test]
    fn uri_path_to_native_windows_drive_letter() {
        let native = PathResolver::uri_path_to_native("/C:/Users/test/file.txt").unwrap();
        assert_eq!(native.as_path(), Path::new(r"C:\Users\test\file.txt"));
    }
}
