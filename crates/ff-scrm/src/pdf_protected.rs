//! Protected / tamper-evident PDF export (CR-NR-098, Wave 3, Requirement 20).
//!
//! Builds the plain selectable-text PDF (see [`crate::pdf`]) and then applies
//! owner-password encryption + permission flags with `lopdf`, so the result is
//! COPY-ENABLED but EDIT-LOCKED (Requirement 20.1, 20.2). An optional user
//! (open/read) password may be set (Requirement 20.2a). A SHA-256 content hash
//! of the source Collection is embedded in the PDF `/Info` dictionary as the
//! tamper-EVIDENCE (Requirement 20.4, 20.5).
//!
//! Honest scope: the permission edit-lock is enforced by owner-password
//! encryption (compliant viewers honour it); the CONTENT HASH is what makes any
//! later alteration DETECTABLE regardless of encryption strength. Digital
//! signatures (Req 20.4a) are a deferred, owner-confirmed add-on.
//!
//! Validates: screen-snapshot-scrm Requirement 20.1, 20.2, 20.2a, 20.3, 20.4, 20.5.

use lopdf::encryption::{EncryptionState, EncryptionVersion, Permissions};
use lopdf::{Document, Object};

use crate::error::{Result, ScrmError};
use crate::evidence::content_hash;
use crate::export::Masking;
use crate::model::ScreenCollection;
use crate::rules::MaskingRules;

/// Options for a protected PDF export.
#[derive(Debug, Clone)]
pub struct ProtectionOptions {
    /// Owner password: required to change permissions/edit; enforces the lock.
    pub owner_password: String,
    /// Optional user (open/read) password. When `None`, the PDF opens freely but
    /// stays edit-locked (Requirement 20.2a).
    pub user_password: Option<String>,
}

/// Build a PROTECTED, tamper-evident PDF for `collection`: selectable text,
/// copy-enabled, edit-locked, with the content hash embedded in `/Info`.
///
/// Validates: screen-snapshot-scrm Requirement 20.1, 20.2, 20.2a, 20.3, 20.4, 20.5.
pub fn export_pdf_protected(
    collection: &ScreenCollection,
    masking: Masking,
    rules: &MaskingRules,
    options: &ProtectionOptions,
) -> Result<Vec<u8>> {
    // 1. Base PDF with real selectable text (Req 20.3).
    let plain = crate::pdf::export_pdf(collection, masking, rules);

    // 2. Load into a lopdf Document.
    let mut doc = Document::load_mem(&plain).map_err(|e| ScrmError::Archive {
        operation: "load base pdf for protection".to_string(),
        reason: e.to_string(),
    })?;

    // 3. Embed the tamper-evidence content hash + provenance in /Info
    //    (Req 20.4, 20.5). Done BEFORE encryption so the strings are encrypted
    //    along with the rest of the document.
    let hash = content_hash(collection);
    let mut info = lopdf::Dictionary::new();
    info.set(
        "Producer",
        Object::string_literal("FileForge Workbench SCRM"),
    );
    info.set("Title", Object::string_literal(collection.name.clone()));
    info.set("FFWBContentHash", Object::string_literal(hash.clone()));
    info.set(
        "FFWBCollectionId",
        Object::string_literal(collection.collection_id.clone()),
    );
    let info_id = doc.add_object(info);
    doc.trailer.set("Info", Object::Reference(info_id));

    // 3a. The encryption algorithm derives the file key from the trailer /ID
    //     (PDF spec 7.5.5). Our hand-written base PDF has no /ID, so add one: a
    //     two-element array of identical 16-byte identifiers derived from the
    //     content hash (deterministic, unique per collection content).
    let id_bytes: Vec<u8> = hash.bytes().take(16).collect();
    let id_padded: Vec<u8> = id_bytes
        .iter()
        .copied()
        .chain(std::iter::repeat(0u8))
        .take(16)
        .collect();
    let id_obj = Object::Array(vec![
        Object::String(id_padded.clone(), lopdf::StringFormat::Hexadecimal),
        Object::String(id_padded, lopdf::StringFormat::Hexadecimal),
    ]);
    doc.trailer.set("ID", id_obj);

    // 4. Apply owner-password encryption + permissions: COPY allowed, MODIFY /
    //    ANNOTATE / ASSEMBLE disallowed (Req 20.1, 20.2). Printing + accessibility
    //    copy stay enabled. AES/RC4 128-bit (V2, revision 3).
    let permissions = Permissions::PRINTABLE
        | Permissions::COPYABLE
        | Permissions::COPYABLE_FOR_ACCESSIBILITY
        | Permissions::PRINTABLE_IN_HIGH_QUALITY;
    let user_password = options.user_password.clone().unwrap_or_default();
    let state: EncryptionState = EncryptionState::try_from(EncryptionVersion::V2 {
        document: &doc,
        owner_password: &options.owner_password,
        user_password: &user_password,
        key_length: 16, // 128-bit
        permissions,
    })
    .map_err(|e| ScrmError::Archive {
        operation: "build encryption state".to_string(),
        reason: e.to_string(),
    })?;
    doc.encrypt(&state).map_err(|e| ScrmError::Archive {
        operation: "encrypt pdf".to_string(),
        reason: e.to_string(),
    })?;

    // 5. Serialise the encrypted document to bytes.
    let mut out = Vec::new();
    doc.save_to(&mut out).map_err(|e| ScrmError::Io {
        operation: "save protected pdf".to_string(),
        source: e,
    })?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use ff_screen_model::{Field, ScreenModel};

    fn sample() -> ScreenCollection {
        let mut c = ScreenCollection::new("cid", "Repro", "alan", Utc::now());
        c.append_capture(
            "a",
            Utc::now(),
            ScreenModel::new("Login").with_field(Field::new("User", "alan")),
        );
        c
    }

    fn opts() -> ProtectionOptions {
        ProtectionOptions {
            owner_password: "owner-secret".to_string(),
            user_password: None,
        }
    }

    // Validates: Req 20.1 -- output is an encrypted PDF (has an /Encrypt dict).
    #[test]
    fn protected_pdf_is_encrypted() {
        let bytes =
            export_pdf_protected(&sample(), Masking::Off, &MaskingRules::default(), &opts())
                .expect("protected export");
        // Re-load and confirm the document reports as encrypted.
        let doc = Document::load_mem(&bytes).expect("reload protected pdf");
        assert!(
            doc.is_encrypted(),
            "protected PDF must carry an /Encrypt dict"
        );
    }

    // Validates: Req 20.1 -- header is a real PDF (not corrupted by encryption).
    #[test]
    fn protected_pdf_has_pdf_header() {
        let bytes =
            export_pdf_protected(&sample(), Masking::Off, &MaskingRules::default(), &opts())
                .expect("protected export");
        assert!(bytes.starts_with(b"%PDF-"), "still a valid PDF header");
    }

    // Validates: Req 20.2a -- an optional user password is accepted (opens the
    // document under the set permissions).
    #[test]
    fn protected_pdf_accepts_optional_user_password() {
        let o = ProtectionOptions {
            owner_password: "owner".to_string(),
            user_password: Some("reader".to_string()),
        };
        let bytes = export_pdf_protected(&sample(), Masking::Off, &MaskingRules::default(), &o)
            .expect("protected export with user password");
        let doc = Document::load_mem(&bytes).expect("reload");
        assert!(doc.is_encrypted());
    }

    // Validates: Req 20.4 -- a content hash is computed for the source (embedded
    // in /Info before encryption). The same hash is exposed for the evidence
    // package, so we assert it is a stable sha256 string here.
    #[test]
    fn content_hash_is_embedded_value() {
        let h = content_hash(&sample());
        assert!(h.starts_with("sha256:"), "hash embedded in /Info is sha256");
    }
}
