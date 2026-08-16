//! Credential types and credential store trait.
//!
//! Defines secure credential types (`SecureString`, `SecureBytes`, `Credential`)
//! and the `CredentialStore` trait for provider-agnostic credential management.
//! Credentials overwrite their backing memory on drop to prevent sensitive data
//! from lingering in freed memory.

use std::fmt;
use std::time::SystemTime;

use crate::error::ConnectorError;

/// A string that overwrites its backing memory on drop.
///
/// Implements a manual zeroize-on-drop pattern to ensure sensitive string data
/// (passwords, tokens) is scrubbed from memory when no longer needed.
///
/// Addresses: Requirement 5 AC 6
#[derive(Clone)]
pub struct SecureString {
    inner: String,
}

impl SecureString {
    /// Creates a new `SecureString` from the given value.
    pub fn new(value: String) -> Self {
        Self { inner: value }
    }

    /// Returns a reference to the inner string value.
    ///
    /// Use sparingly — prefer passing `SecureString` directly to avoid
    /// leaving references to sensitive data on the stack.
    pub fn expose_secret(&self) -> &str {
        &self.inner
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        // Overwrite backing memory with zeros before deallocation.
        // SAFETY: We replace the string content with null bytes of the same length,
        // then clear/shrink to zero capacity.
        let bytes = unsafe { self.inner.as_mut_vec() };
        for byte in bytes.iter_mut() {
            *byte = 0;
        }
        self.inner.clear();
        self.inner.shrink_to_fit();
    }
}

impl fmt::Debug for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecureString(***)")
    }
}

/// A byte buffer that overwrites its backing memory on drop.
///
/// Implements a manual zeroize-on-drop pattern to ensure sensitive byte data
/// (private keys, binary tokens) is scrubbed from memory when no longer needed.
///
/// Addresses: Requirement 5 AC 6
#[derive(Clone)]
pub struct SecureBytes {
    inner: Vec<u8>,
}

impl SecureBytes {
    /// Creates a new `SecureBytes` from the given value.
    pub fn new(value: Vec<u8>) -> Self {
        Self { inner: value }
    }

    /// Returns a reference to the inner byte slice.
    pub fn expose_secret(&self) -> &[u8] {
        &self.inner
    }
}

impl Drop for SecureBytes {
    fn drop(&mut self) {
        // Overwrite backing memory with zeros before deallocation.
        for byte in self.inner.iter_mut() {
            *byte = 0;
        }
        self.inner.clear();
        self.inner.shrink_to_fit();
    }
}

impl fmt::Debug for SecureBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecureBytes(***)")
    }
}

/// A credential for authenticating with a remote service.
///
/// All sensitive fields use secure memory types that zeroize on drop.
/// The enum is `#[non_exhaustive]` to allow adding new authentication
/// methods in future without breaking changes.
///
/// Addresses: Requirement 5 AC 2
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Credential {
    /// Username + password authentication.
    Password {
        /// The username.
        username: String,
        /// The password (zeroized on drop).
        password: SecureString,
    },
    /// SSH key-based authentication.
    KeyBased {
        /// The username.
        username: String,
        /// The private key bytes (zeroized on drop).
        private_key: SecureBytes,
        /// Optional passphrase for the private key.
        passphrase: Option<SecureString>,
    },
    /// OAuth 2.0 token authentication.
    OAuth {
        /// The access token (zeroized on drop).
        access_token: SecureString,
        /// Optional refresh token for renewal.
        refresh_token: Option<SecureString>,
        /// When the access token expires, if known.
        expires_at: Option<SystemTime>,
    },
    /// Bearer/API token authentication.
    Token {
        /// The token value (zeroized on drop).
        token: SecureString,
    },
}

/// Provider-agnostic interface for secure credential management.
///
/// Credentials are scoped by connector scheme and connection name.
/// The key format is typically `"{scheme}:{connection_name}"`.
///
/// Implementors must ensure:
/// - Credentials are never logged in plaintext
/// - Secure memory handling (zeroize on drop)
/// - Proper scoping (no cross-scheme access)
///
/// Addresses: Requirement 5 AC 1, AC 5, AC 6, AC 7
pub trait CredentialStore: Send + Sync {
    /// Store a credential under the given key.
    fn store(&self, key: &str, credential: &Credential) -> Result<(), ConnectorError>;

    /// Retrieve a credential by key. Returns `None` if not found.
    fn retrieve(&self, key: &str) -> Result<Option<Credential>, ConnectorError>;

    /// Delete a stored credential.
    fn delete(&self, key: &str) -> Result<(), ConnectorError>;

    /// Check if a credential exists for the given key.
    fn exists(&self, key: &str) -> bool;

    /// Refresh an expired credential (e.g., OAuth token renewal).
    ///
    /// Addresses: Requirement 5 AC 4
    fn refresh_credential(&self, key: &str) -> Result<Credential, ConnectorError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 5 AC 6
    #[test]
    fn secure_string_debug_masks_value() {
        let secret = SecureString::new("super_secret_password".to_string());
        let debug = format!("{secret:?}");
        assert_eq!(debug, "SecureString(***)");
        assert!(!debug.contains("super_secret_password"));
    }

    // Validates: Requirement 5 AC 6
    #[test]
    fn secure_bytes_debug_masks_value() {
        let secret = SecureBytes::new(vec![1, 2, 3, 4, 5]);
        let debug = format!("{secret:?}");
        assert_eq!(debug, "SecureBytes(***)");
    }

    // Validates: Requirement 5 AC 6
    #[test]
    fn secure_string_expose_secret_returns_value() {
        let secret = SecureString::new("password123".to_string());
        assert_eq!(secret.expose_secret(), "password123");
    }

    // Validates: Requirement 5 AC 6
    #[test]
    fn secure_bytes_expose_secret_returns_value() {
        let data = vec![10, 20, 30];
        let secret = SecureBytes::new(data.clone());
        assert_eq!(secret.expose_secret(), &data);
    }

    // Validates: Requirement 5 AC 2
    #[test]
    fn credential_password_variant_can_be_constructed() {
        let cred = Credential::Password {
            username: "admin".to_string(),
            password: SecureString::new("pass".to_string()),
        };
        assert!(matches!(cred, Credential::Password { .. }));
    }

    // Validates: Requirement 5 AC 2
    #[test]
    fn credential_key_based_variant_can_be_constructed() {
        let cred = Credential::KeyBased {
            username: "user".to_string(),
            private_key: SecureBytes::new(vec![0xDE, 0xAD]),
            passphrase: Some(SecureString::new("phrase".to_string())),
        };
        assert!(matches!(cred, Credential::KeyBased { .. }));
    }

    // Validates: Requirement 5 AC 2
    #[test]
    fn credential_oauth_variant_can_be_constructed() {
        let cred = Credential::OAuth {
            access_token: SecureString::new("access".to_string()),
            refresh_token: Some(SecureString::new("refresh".to_string())),
            expires_at: Some(SystemTime::now()),
        };
        assert!(matches!(cred, Credential::OAuth { .. }));
    }

    // Validates: Requirement 5 AC 2
    #[test]
    fn credential_token_variant_can_be_constructed() {
        let cred = Credential::Token {
            token: SecureString::new("bearer_xyz".to_string()),
        };
        assert!(matches!(cred, Credential::Token { .. }));
    }

    // Validates: Requirement 5 AC 5
    #[test]
    fn credential_debug_does_not_leak_secrets() {
        let cred = Credential::Password {
            username: "admin".to_string(),
            password: SecureString::new("super_secret".to_string()),
        };
        let debug = format!("{cred:?}");
        // Username is ok to show, but password must be masked
        assert!(debug.contains("admin"));
        assert!(!debug.contains("super_secret"));
        assert!(debug.contains("SecureString(***)"));
    }

    // Validates: Requirement 5 AC 1
    #[test]
    fn credential_store_trait_is_object_safe() {
        fn _accept(_: &dyn CredentialStore) {}
        fn _accept_box(_: Box<dyn CredentialStore>) {}
    }
}
