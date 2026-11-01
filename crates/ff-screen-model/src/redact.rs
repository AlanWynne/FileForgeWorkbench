//! Credential-secret redaction as a pure model transform.
//!
//! [`ScreenModel`] carries [`FieldAttributes::sensitive`], but no renderer masks
//! a sensitive value (text/ansi/markdown/html emit it in clear; yaml only adds a
//! `sensitive: true` marker and still prints the value). Applying
//! [`ScreenModel::redacted`] to the model BEFORE any render masks every flavor
//! (text/md/html/ansi/raster/pdf) from one source.
//!
//! Validates: screen-snapshot-scrm Requirement 27.4, 27.5.

use crate::model::ScreenModel;

/// The ASCII placeholder that replaces a sensitive field value.
///
/// Validates: screen-snapshot-scrm Requirement 27.4.
pub const REDACTED_PLACEHOLDER: &str = "[REDACTED]";

impl ScreenModel {
    /// Return a clone of this screen whose every sensitive field value is
    /// replaced by [`REDACTED_PLACEHOLDER`]. Non-sensitive field values, labels,
    /// and attributes are preserved. Applied before render, this masks the
    /// secret in every snapshot flavor from one source.
    ///
    /// Validates: screen-snapshot-scrm Requirement 27.4, 27.5.
    pub fn redacted(&self) -> ScreenModel {
        let mut clone = self.clone();
        for field in &mut clone.fields {
            if field.attrs.sensitive {
                field.value = REDACTED_PLACEHOLDER.to_string();
            }
        }
        clone
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Field, FieldAttributes, ScreenModel};
    use crate::{render, RenderOptions, SnapshotFormat, REDACTED_PLACEHOLDER};

    // Validates: screen-snapshot-scrm Req 27.4, 27.5
    #[test]
    fn redacted_masks_only_sensitive_field_values() {
        let model = ScreenModel::new("Login")
            .with_field(Field::new("User", "alan"))
            .with_field(
                Field::new("Password", "hunter2").with_attrs(FieldAttributes::plain().sensitive()),
            );

        let redacted = model.redacted();

        let password = redacted
            .fields
            .iter()
            .find(|f| f.label == "Password")
            .expect("password field present");
        assert_eq!(password.value, REDACTED_PLACEHOLDER);

        // The secret string must be absent from every field value (27.4).
        assert!(
            redacted.fields.iter().all(|f| !f.value.contains("hunter2")),
            "secret must be absent from the redacted model"
        );

        // The non-secret value is still present (27.5 -- only declared-secret
        // is withheld).
        assert!(
            redacted.fields.iter().any(|f| f.value == "alan"),
            "non-secret value must be preserved"
        );
    }

    // Validates: screen-snapshot-scrm Req 27.4
    #[test]
    fn redacted_model_omits_secret_in_every_text_format() {
        let model = ScreenModel::new("Login")
            .with_field(Field::new("User", "alan"))
            .with_field(
                Field::new("Password", "hunter2").with_attrs(FieldAttributes::plain().sensitive()),
            );
        let redacted = model.redacted();

        for fmt in [
            SnapshotFormat::PlainText,
            SnapshotFormat::Ansi,
            SnapshotFormat::Markdown,
            SnapshotFormat::Html,
            SnapshotFormat::Yaml,
        ] {
            let out = render(&redacted, fmt, RenderOptions::default());
            assert!(
                !out.contains("hunter2"),
                "{fmt:?} output must not contain the secret"
            );
            assert!(
                out.contains(REDACTED_PLACEHOLDER),
                "{fmt:?} output must contain the redaction placeholder"
            );
        }
    }
}
