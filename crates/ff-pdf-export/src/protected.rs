//! Protected / tamper-evident PDF export (feature `protected`).
//!
//! Generic counterpart of the SCRM-specific `ff-scrm::pdf_protected`: it takes
//! ALREADY-BUILT plain PDF bytes (from [`crate::build_pdf_from_pages`] or
//! [`crate::writer::build_pdf`]) plus optional provenance metadata, and applies
//! owner-password encryption + permission flags with `lopdf`, so the result is
//! COPY-ENABLED but EDIT-LOCKED. An optional user (open/read) password may be
//! set. When a `content_hash` is supplied it is embedded in the PDF `/Info`
//! dictionary as tamper-EVIDENCE (embedded before encryption).
//!
//! Honest scope: the permission edit-lock is enforced by owner-password
//! encryption (compliant viewers honour it); the CONTENT HASH is what makes any
//! later alteration DETECTABLE. This crate does not compute the hash -- the
//! caller supplies one (e.g. a SHA-256 of the source), keeping this module free
//! of any domain-specific hashing dependency.

use anyhow::{Context, Result};
use lopdf::encryption::{EncryptionState, EncryptionVersion, Permissions};
use lopdf::{Document, Object};

/// Provenance metadata embedded in the PDF `/Info` dictionary before encryption.
#[derive(Debug, Clone, Default)]
pub struct PdfInfo {
    /// Document title (embedded as `/Title`).
    pub title: Option<String>,
    /// Tamper-evidence content hash (embedded as `/FFWBContentHash`). The caller
    /// computes this over the source content; `None` skips the field.
    pub content_hash: Option<String>,
    /// Optional source identifier (embedded as `/FFWBSourceId`).
    pub source_id: Option<String>,
}

/// Options for a protected PDF export.
#[derive(Debug, Clone)]
pub struct ProtectionOptions {
    /// Owner password: required to change permissions/edit; enforces the lock.
    pub owner_password: String,
    /// Optional user (open/read) password. When `None`, the PDF opens freely but
    /// stays edit-locked.
    pub user_password: Option<String>,
}

/// Apply owner-password encryption + permissions to already-built plain PDF
/// bytes, embedding the supplied provenance metadata in `/Info` first.
///
/// Returns the encrypted PDF bytes: selectable text, copy-enabled, edit-locked.
///
/// # Errors
///
/// Returns an error if the plain bytes cannot be parsed as a PDF, the encryption
/// state cannot be built, or the encrypted document cannot be serialised.
pub fn protect_pdf(plain: &[u8], info: &PdfInfo, options: &ProtectionOptions) -> Result<Vec<u8>> {
    // 1. Load the plain PDF into a lopdf Document.
    let mut doc = Document::load_mem(plain).context("load base pdf for protection")?;

    // 2. Embed provenance in /Info BEFORE encryption so the strings are
    //    encrypted along with the rest of the document.
    let mut info_dict = lopdf::Dictionary::new();
    info_dict.set(
        "Producer",
        Object::string_literal("FileForge Workbench PDF Export"),
    );
    if let Some(title) = &info.title {
        info_dict.set("Title", Object::string_literal(title.clone()));
    }
    if let Some(hash) = &info.content_hash {
        info_dict.set("FFWBContentHash", Object::string_literal(hash.clone()));
    }
    if let Some(sid) = &info.source_id {
        info_dict.set("FFWBSourceId", Object::string_literal(sid.clone()));
    }
    let info_id = doc.add_object(info_dict);
    doc.trailer.set("Info", Object::Reference(info_id));

    // 2a. The encryption algorithm derives the file key from the trailer /ID
    //     (PDF spec 7.5.5). The hand-written base PDF has no /ID, so add one: a
    //     16-byte identifier derived from the content hash when present,
    //     otherwise a fixed pad (deterministic).
    let seed: Vec<u8> = info
        .content_hash
        .as_deref()
        .unwrap_or("FileForgeWorkbench")
        .bytes()
        .collect();
    let id_padded: Vec<u8> = seed
        .into_iter()
        .chain(std::iter::repeat(0u8))
        .take(16)
        .collect();
    let id_obj = Object::Array(vec![
        Object::String(id_padded.clone(), lopdf::StringFormat::Hexadecimal),
        Object::String(id_padded, lopdf::StringFormat::Hexadecimal),
    ]);
    doc.trailer.set("ID", id_obj);

    // 3. Apply owner-password encryption + permissions: COPY allowed, MODIFY /
    //    ANNOTATE / ASSEMBLE disallowed. Printing + accessibility copy stay
    //    enabled. AES/RC4 128-bit (V2, revision 3).
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
    .map_err(|e| anyhow::anyhow!("build encryption state: {e}"))?;
    doc.encrypt(&state)
        .map_err(|e| anyhow::anyhow!("encrypt pdf: {e}"))?;

    // 4. Serialise the encrypted document to bytes.
    let mut out = Vec::new();
    doc.save_to(&mut out).context("save protected pdf")?;
    Ok(out)
}

/// Build a plain PDF from `pages` and then protect it in one call.
///
/// # Errors
///
/// See [`protect_pdf`].
pub fn export_pdf_protected(
    pages: &[Vec<String>],
    info: &PdfInfo,
    options: &ProtectionOptions,
) -> Result<Vec<u8>> {
    let plain = crate::writer::build_pdf(pages);
    protect_pdf(&plain, info, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pages() -> Vec<Vec<String>> {
        vec![vec!["Login".to_string(), "User: alan".to_string()]]
    }

    fn opts() -> ProtectionOptions {
        ProtectionOptions {
            owner_password: "owner-secret".to_string(),
            user_password: None,
        }
    }

    /// Output is an encrypted PDF (reloads as encrypted).
    #[test]
    fn protected_pdf_is_encrypted() {
        let info = PdfInfo {
            title: Some("Repro".to_string()),
            content_hash: Some("sha256:abc123".to_string()),
            source_id: None,
        };
        let bytes = export_pdf_protected(&pages(), &info, &opts()).expect("protected export");
        let doc = Document::load_mem(&bytes).expect("reload protected pdf");
        assert!(
            doc.is_encrypted(),
            "protected PDF must carry an /Encrypt dict"
        );
    }

    /// Still a valid PDF header after encryption.
    #[test]
    fn protected_pdf_has_pdf_header() {
        let bytes =
            export_pdf_protected(&pages(), &PdfInfo::default(), &opts()).expect("protected export");
        assert!(bytes.starts_with(b"%PDF-"), "still a valid PDF header");
    }

    /// An optional user password is accepted.
    #[test]
    fn protected_pdf_accepts_optional_user_password() {
        let o = ProtectionOptions {
            owner_password: "owner".to_string(),
            user_password: Some("reader".to_string()),
        };
        let bytes = export_pdf_protected(&pages(), &PdfInfo::default(), &o)
            .expect("protected export with user password");
        let doc = Document::load_mem(&bytes).expect("reload");
        assert!(doc.is_encrypted());
    }

    /// Protecting already-built bytes works via `protect_pdf` directly.
    #[test]
    fn protect_pdf_encrypts_prebuilt_bytes() {
        let plain = crate::writer::build_pdf(&pages());
        let bytes = protect_pdf(&plain, &PdfInfo::default(), &opts()).expect("protect");
        assert!(Document::load_mem(&bytes).expect("reload").is_encrypted());
    }
}
