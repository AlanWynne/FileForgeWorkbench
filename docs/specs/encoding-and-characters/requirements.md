# Requirements Document

## Introduction

This feature specifies the **Encoding and Characters** subsystem for FileForgeWorkbench — the `ff-encoding` crate. This crate provides encoding detection, BOM (Byte Order Mark) handling, encoding conversion on load and save, word-character classification (extending Scintilla's `CharClassify` to full Unicode), grapheme cluster boundary detection, and DBCS (Double-Byte Character Set) support for legacy East Asian encodings.

The encoding crate is **GUI-independent** — it has no rendering or framework dependency. It operates as a service layer consumed by the document model during file loading/saving and by the find-and-replace engine for Unicode case folding and word boundary detection.

This specification is derived primarily from Scintilla's character-handling infrastructure:

- **Scintilla Document Requirement 11** (Character and Encoding Navigation): UTF-8 navigation, DBCS lead/trail byte detection, encoding-family classification
- **Scintilla CharClassify**: ASCII-range character classification into space/newLine/word/punctuation classes with configurable word-character sets
- **Scintilla CharacterCategoryMap**: Unicode General Category lookup (Lu, Ll, Nd, etc.) for full-Unicode word classification
- **Scintilla UniConversion**: UTF-8/UTF-16/UTF-32 conversion, UTF-8 validation, replacement character handling
- **Scintilla DBCS**: Shift-JIS (CP932), GBK (CP936), Korean Wansung (CP949), Big5 (CP950), Johab (CP1361) lead/trail byte detection
- **Scintilla CaseConvert**: Unicode case folding and conversion (fold, upper, lower)

**Source references:**
- **[SCI-DOC-11]** = Scintilla document-cellbuffer Requirement 11: Character and Encoding Navigation
- **[SCI-CHAR]** = Scintilla CharClassify + CharacterCategoryMap + CharacterType
- **[SCI-UNI]** = Scintilla UniConversion (UTF-8/16/32 conversion, validation)
- **[SCI-DBCS]** = Scintilla DBCS module (lead/trail byte, code pages, fold maps)
- **[SCI-CASE]** = Scintilla CaseConvert (Unicode case folding)
- **[WB]** = Workbench Platform Architecture Brief
- **[FFE]** = FileForgeEditor specifications (encoding detection on open/save)

## Cross-References

- **`document-model`** — The document model stores text as UTF-8 internally; encoding conversion happens at the boundary (load/save). Character navigation methods in document-model delegate to this crate for encoding awareness. [SCI-DOC-11]
- **`file-operations`** — File open and save operations use this crate to detect encoding on load and convert on save. BOM decisions are made during save. [FFE]
- **`find-and-replace`** — Unicode case folding for case-insensitive search delegates to this crate's `CaseFolder`. Word-boundary detection uses this crate's character classification. [SCI-CASE]
- **`edit-operations`** — Word-selection (double-click), word-delete, and Ctrl+arrow word navigation depend on word-character classification from this crate. [SCI-CHAR]
- **`navigation-commands`** — Word-part navigation (camelCase/snake_case sub-word movement) uses this crate's `WordPartSeparator` classification. [SCI-DOC-11]
- **`fileforge-integration`** — EBCDIC encoding support for mainframe file formats. [FFE]
- **`background-io`** — Encoding detection may run as part of async file loading pipeline. [WB]

---

## Glossary

- **BOM (Byte Order Mark)**: A Unicode character (U+FEFF) placed at the start of a file to indicate encoding and byte order. UTF-8 BOM = `EF BB BF`, UTF-16LE BOM = `FF FE`, UTF-16BE BOM = `FE FF`, UTF-32LE BOM = `FF FE 00 00`, UTF-32BE BOM = `00 00 FE FF`. [SCI-UNI]
- **CharClassify**: A 256-entry lookup table classifying each byte value (0–255) into one of four classes: Space, NewLine, Word, or Punctuation. Configurable word-character sets allow adding characters to the Word class. [SCI-CHAR]
- **CharacterCategory**: A Unicode General Category classification (Lu, Ll, Lt, Lm, Lo, Mn, Mc, Me, Nd, Nl, No, Pc, Pd, Ps, Pe, Pi, Pf, Po, Sm, Sc, Sk, So, Zs, Zl, Zp, Cc, Cf, Cs, Co, Cn) used for full-Unicode word classification beyond the ASCII range. [SCI-CHAR]
- **CharacterCategoryMap**: An optimized lookup structure mapping Unicode code points to their General Category, using a dense array for common characters and binary search for rare ones. [SCI-CHAR]
- **DBCS (Double-Byte Character Set)**: Legacy East Asian encodings where characters are represented by either one or two bytes. Includes Shift-JIS (CP932), GBK (CP936), Korean Wansung (CP949), Big5 (CP950), and Johab (CP1361). [SCI-DBCS]
- **EncodingFamily**: An enum categorising encodings into families: SingleByte (ASCII, ISO-8859-x), UTF-8, DBCS (Shift-JIS, GBK, etc.), and Unicode (UTF-16, UTF-32). [SCI-DOC-11]
- **GraphemeCluster**: A user-perceived character that may span multiple Unicode code points — e.g., base character + combining marks, regional indicator pairs (flags), emoji ZWJ sequences. [WB]
- **CaseFolder**: A component performing Unicode case folding for case-insensitive comparison. Supports fold (for search), upper, and lower conversions. Not locale-sensitive. [SCI-CASE]
- **LeadByte**: In DBCS encodings, the first byte of a two-byte character sequence. The valid ranges differ per code page. [SCI-DBCS]
- **TrailByte**: In DBCS encodings, the second byte of a two-byte character sequence. [SCI-DBCS]
- **ReplacementCharacter**: U+FFFD, used to represent invalid or undecodable byte sequences during encoding conversion. [SCI-UNI]

---

## Requirements

### Requirement 1: Encoding Detection

**User Story:** As a file-loading component, I want automatic encoding detection from file content, so that files are correctly decoded regardless of whether they carry a BOM or metadata indicating their encoding.

**Source:** [SCI-DOC-11], [SCI-UNI], [FFE]

#### Acceptance Criteria

1. WHEN a file is opened without explicit encoding specification, THE encoding detector SHALL examine the first N bytes (configurable, default 8192) to determine the file's encoding. [SCI-DOC-11, FFE]
2. THE encoding detector SHALL support detection of the following encodings: UTF-8 (with and without BOM), UTF-16LE, UTF-16BE, UTF-32LE, UTF-32BE, ISO-8859-1 (Latin-1), Windows-1252, Shift-JIS (CP932), GBK (CP936), EUC-KR (CP949), Big5 (CP950), and EBCDIC (CP037/CP500). [SCI-DOC-11, SCI-DBCS, FFE]
3. THE encoding detector SHALL use the following priority order: (a) BOM presence, (b) UTF-8 validity check, (c) DBCS lead/trail byte pattern analysis, (d) statistical byte-frequency heuristics, (e) fallback to configurable default encoding. [SCI-UNI]
4. WHEN the file content is valid UTF-8 (all multi-byte sequences conform to RFC 3629), THE detector SHALL classify it as UTF-8 unless a non-UTF-8 BOM is present. [SCI-UNI]
5. IF the file content contains null bytes in a pattern consistent with UTF-16 or UTF-32 (alternating nulls for UTF-16, triple nulls for UTF-32), THEN THE detector SHALL classify accordingly based on byte-order patterns. [SCI-UNI]
6. THE encoding detector SHALL return a confidence level (High, Medium, Low) alongside the detected encoding, enabling the caller to prompt the user when confidence is low. [WB]
7. THE encoding detector SHALL be stateless and side-effect-free — it SHALL operate on a byte slice without modifying any document state. [WB]
8. WHEN an explicit encoding is provided by the user or configuration (e.g., per-file or per-project encoding override), THE system SHALL skip detection and use the specified encoding directly. [FFE]

---

### Requirement 2: BOM Detection and Handling

**User Story:** As a file-loading component, I want BOM detection and configurable BOM handling on save, so that files with BOMs are loaded correctly and the user can control whether a BOM is written on save.

**Source:** [SCI-UNI], [FFE]

#### Acceptance Criteria

1. WHEN the first bytes of a file match a known BOM sequence, THE BOM detector SHALL identify the encoding and report the BOM length (in bytes) so that the BOM can be excluded from document content. [SCI-UNI]
2. THE BOM detector SHALL recognise the following sequences: UTF-8 (`EF BB BF`, 3 bytes), UTF-16LE (`FF FE`, 2 bytes), UTF-16BE (`FE FF`, 2 bytes), UTF-32LE (`FF FE 00 00`, 4 bytes), UTF-32BE (`00 00 FE FF`, 4 bytes). [SCI-UNI]
3. WHEN a UTF-32LE BOM (`FF FE 00 00`) is detected, THE detector SHALL distinguish it from a UTF-16LE BOM (`FF FE`) followed by a NUL character by checking the full 4-byte prefix first. [SCI-UNI]
4. THE Document SHALL store whether a BOM was present in the original file as metadata (`has_bom: bool`) associated with the document's encoding state. [FFE]
5. WHEN saving a file, IF the document's `has_bom` metadata is `true`, THEN THE encoder SHALL prepend the appropriate BOM for the target encoding. [FFE]
6. WHEN saving a file, IF the user explicitly requests BOM removal or addition (via command or configuration), THEN THE encoder SHALL honour that request regardless of the `has_bom` metadata. [FFE]
7. IF the target save encoding is UTF-8 and no BOM preference is set, THEN THE default behaviour SHALL be to preserve the original BOM state (write BOM if file had BOM, omit if it did not). [FFE]
8. THE BOM detector SHALL be invocable independently of full encoding detection — callers SHALL be able to check for BOM presence without triggering heuristic analysis. [WB]

---

### Requirement 3: Encoding Conversion on Load

**User Story:** As a document model, I want source-encoding-to-UTF-8 conversion during file loading, so that all document content is stored uniformly as UTF-8 internally regardless of the original file encoding.

**Source:** [SCI-UNI], [SCI-DBCS], [WB]

#### Acceptance Criteria

1. WHEN a file is loaded with an encoding other than UTF-8, THE converter SHALL transcode the byte content to valid UTF-8 before storing it in the document buffer. [SCI-UNI]
2. THE converter SHALL support transcoding from all detected encodings (Requirement 1.2) to UTF-8, including: UTF-16LE/BE, UTF-32LE/BE, ISO-8859-1, Windows-1252, Shift-JIS, GBK, EUC-KR, Big5, and EBCDIC code pages. [SCI-UNI, SCI-DBCS]
3. WHEN an invalid or unmappable byte sequence is encountered during conversion, THE converter SHALL replace it with the Unicode Replacement Character (U+FFFD) and record the position and original bytes in a conversion-issues log. [SCI-UNI]
4. THE conversion-issues log SHALL be accessible after loading completes, enabling the UI to warn the user about lossy conversion. [WB]
5. WHEN converting from UTF-16 or UTF-32, THE converter SHALL correctly handle surrogate pairs (UTF-16) and supplementary plane characters, producing valid 4-byte UTF-8 sequences for code points above U+FFFF. [SCI-UNI]
6. WHEN converting from a DBCS encoding, THE converter SHALL use the code-page-specific lead/trail byte tables to correctly segment multi-byte characters before mapping to Unicode code points. [SCI-DBCS]
7. THE converter SHALL preserve the exact byte count of each source line after conversion, maintaining a mapping from source byte offsets to UTF-8 byte offsets for diagnostic purposes. [WB]
8. THE conversion process SHALL be streaming-capable — it SHALL process chunks of input without requiring the entire file in memory, enabling integration with the background-io pipeline. [WB]

---

### Requirement 4: Encoding Conversion on Save

**User Story:** As a file-saving component, I want UTF-8-to-target-encoding conversion during save, so that files can be saved in their original encoding or a user-specified encoding while the document remains UTF-8 internally.

**Source:** [SCI-UNI], [SCI-DBCS], [FFE]

#### Acceptance Criteria

1. WHEN a file is saved, THE converter SHALL transcode the document's UTF-8 content to the target encoding specified in the document's encoding metadata. [SCI-UNI]
2. THE default target encoding for save SHALL be the encoding detected (or specified) when the file was loaded — preserving the original encoding unless the user explicitly changes it. [FFE]
3. WHEN the user requests "Save As" with a different encoding, THE converter SHALL transcode to the new encoding and update the document's encoding metadata to reflect the change. [FFE]
4. WHEN a Unicode character in the document has no representation in the target encoding (unmappable character), THE converter SHALL report the character position and code point as a save-encoding error, and SHALL NOT silently discard or corrupt the character. [WB]
5. IF unmappable characters are detected during save-encoding, THEN THE system SHALL present the user with options: (a) abort save, (b) replace unmappable characters with a placeholder (e.g., `?`), or (c) switch to UTF-8 encoding for the save. [WB]
6. WHEN converting to UTF-16LE/BE or UTF-32LE/BE, THE converter SHALL produce valid surrogate pairs (UTF-16) or direct code point values (UTF-32) for supplementary plane characters. [SCI-UNI]
7. WHEN converting to a DBCS encoding, THE converter SHALL use the encoding's Unicode-to-byte mapping table to produce correct lead+trail byte sequences. [SCI-DBCS]
8. THE save-conversion process SHALL be streaming-capable — it SHALL produce output chunks suitable for async write without buffering the entire converted file in memory. [WB]

---

### Requirement 5: UTF-8 Validation and Repair

**User Story:** As a document model consumer, I want UTF-8 validation and repair utilities, so that invalid byte sequences in document content are detected and can be corrected without data loss.

**Source:** [SCI-UNI], [SCI-DOC-11]

#### Acceptance Criteria

1. WHEN `utf8_validate(bytes)` is called, THE validator SHALL return `true` if the entire byte slice is valid UTF-8 per RFC 3629 (no overlong encodings, no surrogates U+D800–U+DFFF, no code points above U+10FFFF). [SCI-UNI]
2. WHEN `utf8_classify(bytes)` is called on a byte sequence, THE classifier SHALL return the byte length of the first character (1–4) and a validity flag. Invalid sequences SHALL be reported with their expected vs actual byte count. [SCI-UNI]
3. WHEN `utf8_fix_invalid(text)` is called, THE repair function SHALL replace each invalid byte sequence with U+FFFD (Replacement Character), preserving all valid UTF-8 content unchanged. [SCI-UNI]
4. THE UTF-8 classifier SHALL correctly identify trail bytes (0x80–0xBF), ASCII bytes (0x00–0x7F), and lead bytes (0xC2–0xF4), rejecting invalid lead byte values (0xC0, 0xC1, 0xF5–0xFF). [SCI-UNI]
5. THE UTF-8 validator SHALL detect and reject overlong encodings — sequences that use more bytes than necessary to encode a code point (e.g., 0xC0 0x80 for NUL). [SCI-UNI]
6. WHEN `utf8_byte_length_from_lead(byte)` is called with a lead byte, THE function SHALL return the expected sequence length (1 for ASCII, 2–4 for multi-byte), or 1 for invalid lead bytes (treating them as single-byte replacement targets). [SCI-UNI]
7. THE validator SHALL handle the Unicode line separator (U+2028), paragraph separator (U+2029), and NEL (U+0085) as valid UTF-8 sequences — they are valid characters that may or may not be treated as line endings depending on the document's LineEndMode. [SCI-UNI]

---

### Requirement 6: Word-Character Classification (CharClassify)

**User Story:** As a word-navigation and selection system, I want configurable character classification that distinguishes word characters from punctuation and whitespace, so that double-click selection, Ctrl+arrow movement, and word-based find operations work correctly for all character types.

**Source:** [SCI-CHAR], [SCI-DOC-11]

#### Acceptance Criteria

1. THE CharClassify component SHALL maintain a 256-entry lookup table mapping each byte value (0x00–0xFF) to one of four classes: `Space`, `NewLine`, `Word`, or `Punctuation`. [SCI-CHAR]
2. WHEN `set_default_char_classes(include_word_class)` is called with `true`, THE classifier SHALL initialise with: CR/LF as NewLine, control characters and space as Space, alphanumeric plus underscore plus bytes 0x80–0xFF as Word, and remaining ASCII punctuation as Punctuation. [SCI-CHAR]
3. WHEN `set_default_char_classes(include_word_class)` is called with `false`, THE classifier SHALL initialise with all non-space, non-newline characters as Punctuation (no Word class), enabling raw character-by-character navigation. [SCI-CHAR]
4. WHEN `set_char_classes(chars, class)` is called, THE classifier SHALL update the classification for each byte in `chars` to the specified class, enabling user-configurable word-character sets (e.g., adding `$` or `#` as word characters for specific languages). [SCI-CHAR]
5. WHEN `classify(byte)` is called, THE classifier SHALL return the CharacterClass for that byte value via O(1) array lookup. [SCI-CHAR]
6. WHEN `is_word(byte)` is called, THE classifier SHALL return `true` if the byte's class is `Word`, providing a fast predicate for word-boundary detection. [SCI-CHAR]
7. WHEN `get_chars_of_class(class)` is called, THE classifier SHALL return a list of all byte values currently assigned to the specified class. [SCI-CHAR]
8. THE CharClassify configuration SHALL be stored per-document, enabling different word-character sets for different file types or user preferences. [SCI-CHAR]

---

### Requirement 7: Unicode Character Category Classification

**User Story:** As a word-navigation system working with Unicode text beyond ASCII, I want Unicode General Category lookup for any code point, so that word boundaries are correctly identified for text in all scripts (Latin, CJK, Cyrillic, Arabic, etc.).

**Source:** [SCI-CHAR], [SCI-DOC-11]

#### Acceptance Criteria

1. THE CharacterCategoryMap SHALL provide a `category_for(code_point)` method that returns the Unicode General Category (one of 30 categories: Lu, Ll, Lt, Lm, Lo, Mn, Mc, Me, Nd, Nl, No, Pc, Pd, Ps, Pe, Pi, Pf, Po, Sm, Sc, Sk, So, Zs, Zl, Zp, Cc, Cf, Cs, Co, Cn) for any valid Unicode code point (0–0x10FFFF). [SCI-CHAR]
2. THE CharacterCategoryMap SHALL use a dense array for code points in the Basic Multilingual Plane (U+0000–U+FFFF) and a binary search over ranges for supplementary plane characters, balancing memory usage with lookup speed. [SCI-CHAR]
3. WHEN `optimize(count_characters)` is called, THE map SHALL pre-allocate the dense array up to `count_characters` entries, trading memory for O(1) lookup speed for the specified range. [SCI-CHAR]
4. THE system SHALL provide `is_id_start(code_point)` and `is_id_continue(code_point)` predicates implementing UAX #31 default identifier rules — identifying characters valid at the start of an identifier vs continuation positions. [SCI-CHAR]
5. THE system SHALL provide `is_xid_start(code_point)` and `is_xid_continue(code_point)` predicates implementing UAX #31 extended identifier rules (XID_Start, XID_Continue properties). [SCI-CHAR]
6. WHEN determining word boundaries for Unicode text, THE system SHALL classify characters into word-like (categories L*, Nd, Nl, Pc — letters, decimal digits, letter-numbers, connector punctuation) and non-word categories, enabling correct word selection across all scripts. [SCI-CHAR]
7. THE CharacterCategoryMap data SHALL be generated from the Unicode Character Database and SHALL be updatable when new Unicode versions are released (via a build-time generation script). [SCI-CHAR]

---

### Requirement 8: DBCS (Double-Byte Character Set) Support

**User Story:** As a developer working with legacy East Asian encoded files, I want correct handling of DBCS encodings including lead/trail byte detection and safe character segmentation, so that files in Shift-JIS, GBK, Big5, and Korean encodings are handled without character corruption.

**Source:** [SCI-DBCS], [SCI-DOC-11]

#### Acceptance Criteria

1. THE DBCS module SHALL support the following code pages: Shift-JIS (CP932), GBK (CP936), Korean Wansung (CP949), Big5 (CP950), and Korean Johab (CP1361). [SCI-DBCS]
2. WHEN `is_dbcs_code_page(code_page)` is called, THE function SHALL return `true` for the five supported DBCS code pages and `false` for all others. [SCI-DBCS]
3. WHEN `dbcs_is_lead_byte(code_page, byte)` is called, THE function SHALL return `true` if the byte falls within the lead-byte range for the specified code page (e.g., 0x81–0x9F and 0xE0–0xFC for Shift-JIS). [SCI-DBCS]
4. WHEN `dbcs_is_trail_byte(code_page, byte)` is called, THE function SHALL return `true` if the byte falls within the trail-byte range for the specified code page (e.g., 0x40–0xFC excluding 0x7F for Shift-JIS). [SCI-DBCS]
5. WHEN `is_dbcs_valid_single_byte(code_page, byte)` is called, THE function SHALL return `true` for bytes that are valid single-byte characters in the DBCS encoding (e.g., 0x80 and 0xA0–0xDF for Shift-JIS half-width katakana). [SCI-DBCS]
6. THE DBCS module SHALL provide a `safe_segment(text, code_page)` function that returns the longest prefix of `text` that ends on a character boundary (not splitting a lead+trail pair), enabling safe text segmentation for display and editing. [SCI-DOC-11]
7. THE DBCS module SHALL provide case-folding maps per code page (`DBCSFoldMap`) that map double-byte characters to their case-folded equivalents for case-insensitive search in DBCS-encoded content. [SCI-DBCS]
8. WHEN the document's encoding family is DBCS, THE character-navigation functions (next_position, char_length_at) SHALL use the DBCS lead/trail byte logic instead of UTF-8 sequence detection to determine character boundaries. [SCI-DOC-11]

---

### Requirement 9: Grapheme Cluster Boundaries

**User Story:** As a caret-movement and selection system, I want grapheme cluster boundary detection, so that the caret moves over user-perceived characters as atomic units — including combining marks, emoji sequences, and regional indicators.

**Source:** [WB], [SCI-DOC-11]

#### Acceptance Criteria

1. THE grapheme cluster detector SHALL implement Unicode UAX #29 (Text Segmentation) grapheme cluster boundary rules, treating combining character sequences as single units for navigation purposes. [WB]
2. WHEN `is_grapheme_boundary(text, byte_offset)` is called, THE detector SHALL return `true` if the byte offset falls on a grapheme cluster boundary according to UAX #29 rules. [WB]
3. WHEN `next_grapheme_boundary(text, byte_offset)` is called, THE detector SHALL return the byte offset of the next grapheme cluster boundary after the given position. [WB]
4. WHEN `prev_grapheme_boundary(text, byte_offset)` is called, THE detector SHALL return the byte offset of the previous grapheme cluster boundary before the given position. [WB]
5. THE detector SHALL handle combining mark sequences (base character + one or more Mn/Mc category characters) as a single grapheme cluster — the caret SHALL NOT land between a base character and its combining marks. [WB]
6. THE detector SHALL handle emoji modifier sequences (emoji + skin tone modifier), emoji ZWJ sequences (emoji + ZWJ + emoji), and regional indicator pairs (flag sequences) as single grapheme clusters. [WB]
7. THE detector SHALL handle Hangul syllable sequences (L* V* T*) as single grapheme clusters per UAX #29 rules. [WB]
8. THE grapheme cluster boundary detection SHALL be configurable at the document level — documents MAY opt into strict grapheme clustering (full UAX #29) or simplified mode (code-point-level navigation only, for performance with very large files). [WB]

---

### Requirement 10: Unicode Case Folding and Conversion

**User Story:** As a search engine and text-processing component, I want Unicode case folding and case conversion, so that case-insensitive search works correctly across all Unicode scripts and case transformation commands (UPPER/LOWER) handle multi-byte expansions.

**Source:** [SCI-CASE], [SCI-DOC-11]

#### Acceptance Criteria

1. THE CaseFolder SHALL support three conversion modes: `Fold` (for case-insensitive comparison), `Upper` (to uppercase), and `Lower` (to lowercase). [SCI-CASE]
2. WHEN `case_convert(code_point, mode)` is called, THE converter SHALL return the UTF-8 byte sequence for the converted character, or an empty result if no conversion applies. [SCI-CASE]
3. WHEN `case_convert_string(text, mode)` is called, THE converter SHALL return a new string with all characters converted according to the specified mode, handling multi-byte expansion (converted string may be up to 3× longer than input). [SCI-CASE]
4. THE case folding data SHALL be derived from the Unicode CaseFolding.txt database (full case folding, status C+F), enabling correct comparison of characters like ß (folds to "ss"), ﬁ (folds to "fi"), and ΐ (folds to ι + combining marks). [SCI-CASE]
5. THE case conversion SHALL NOT be locale-sensitive — it SHALL use the default Unicode mappings regardless of system locale. Locale-sensitive operations (e.g., Turkish İ/ı) are explicitly out of scope. [SCI-CASE]
6. WHEN case folding produces a multi-character expansion (e.g., ß → ss), THE fold result SHALL be the expanded form for comparison purposes. [SCI-CASE]
7. THE CaseFolder SHALL provide a `ICaseConverter` trait with a `case_convert_string` method, enabling the find-and-replace engine to use case folding without depending on the specific implementation. [SCI-CASE]
8. THE case conversion tables SHALL be generated at build time from Unicode data files and compiled into the crate as static data, avoiding runtime file loading. [SCI-CASE]

---

### Requirement 11: Encoding Family Classification

**User Story:** As a document model, I want to classify the active encoding into families, so that character navigation and line-end detection can select the appropriate algorithm (UTF-8 sequence parsing vs DBCS lead/trail vs single-byte lookup).

**Source:** [SCI-DOC-11]

#### Acceptance Criteria

1. THE system SHALL define an `EncodingFamily` enum with variants: `SingleByte` (ASCII, ISO-8859-x, Windows-125x, EBCDIC), `Utf8`, `Dbcs` (Shift-JIS, GBK, Big5, Korean), and `Utf16` (for internal processing of UTF-16 streams before conversion). [SCI-DOC-11]
2. WHEN `encoding_family(code_page)` is called, THE function SHALL return the appropriate EncodingFamily for the given code page identifier. [SCI-DOC-11]
3. WHEN the encoding family is `Utf8`, THE character navigation functions SHALL use UTF-8 lead-byte/trail-byte logic to determine character boundaries (1–4 bytes per character). [SCI-DOC-11]
4. WHEN the encoding family is `Dbcs`, THE character navigation functions SHALL use code-page-specific lead/trail byte ranges to determine character boundaries (1–2 bytes per character). [SCI-DOC-11]
5. WHEN the encoding family is `SingleByte`, THE character navigation functions SHALL treat each byte as one character (1:1 byte-to-character mapping). [SCI-DOC-11]
6. THE encoding family SHALL be queryable from the Document to allow rendering, selection, and editing components to adapt their behaviour based on the active encoding model. [SCI-DOC-11]
7. WHEN `set_encoding(code_page)` is called on a Document, THE system SHALL update the encoding family and trigger re-classification of any encoding-dependent data structures (e.g., DBCS fold maps). [SCI-DOC-11]

---

### Requirement 12: Word-Part Navigation Support

**User Story:** As a developer using camelCase or snake_case identifiers, I want sub-word boundary detection, so that Ctrl+Left/Right can move through identifier parts (e.g., stopping at each capital letter in "getDocumentModel" or at each underscore in "get_document_model").

**Source:** [SCI-DOC-11], [SCI-CHAR]

#### Acceptance Criteria

1. THE system SHALL provide a `is_word_part_separator(code_point)` predicate that returns `true` for characters that form sub-word boundaries within identifiers: underscores, transitions from lowercase to uppercase, transitions from uppercase sequence to lowercase (e.g., "XML" to "Parser" in "XMLParser"). [SCI-DOC-11]
2. WHEN `word_part_left(text, position)` is called, THE function SHALL return the byte position of the beginning of the previous word-part to the left of `position`, respecting camelCase and snake_case boundaries. [SCI-DOC-11]
3. WHEN `word_part_right(text, position)` is called, THE function SHALL return the byte position of the beginning of the next word-part to the right of `position`. [SCI-DOC-11]
4. THE word-part boundaries SHALL include: (a) underscore characters, (b) transitions from lowercase letter to uppercase letter, (c) transitions from a run of uppercase letters to a lowercase letter (placing boundary before the last uppercase), (d) transitions between letter and digit. [SCI-DOC-11]
5. THE word-part navigation SHALL respect the document's CharClassify configuration — characters not classified as Word SHALL act as hard word-part boundaries (the word-part does not span across punctuation or space). [SCI-CHAR]
6. THE word-part functions SHALL work with Unicode text, treating any Lu/Ll category transition as a camelCase boundary, not just ASCII A-Z/a-z. [SCI-CHAR]

---

### Requirement 13: Configurable Word-Character Sets

**User Story:** As a user working in different programming languages, I want to configure which characters are treated as word characters, so that word selection and navigation match my language's identifier rules (e.g., `$` in PHP/Perl, `#` in C preprocessor directives, `-` in Lisp).

**Source:** [SCI-CHAR], [WB]

#### Acceptance Criteria

1. THE system SHALL provide per-document word-character configuration that can be set via the configuration system (TOML language definitions) or programmatically by plugins. [SCI-CHAR, WB]
2. WHEN `set_word_chars(chars)` is called, THE classifier SHALL add all specified characters to the Word class, overriding their default classification. [SCI-CHAR]
3. WHEN `set_whitespace_chars(chars)` is called, THE classifier SHALL add all specified characters to the Space class. [SCI-CHAR]
4. WHEN `set_punctuation_chars(chars)` is called, THE classifier SHALL add all specified characters to the Punctuation class. [SCI-CHAR]
5. WHEN `reset_word_chars()` is called, THE classifier SHALL restore the default character classification (alphanumeric + underscore + bytes ≥0x80 as Word). [SCI-CHAR]
6. THE word-character configuration SHALL affect all word-based operations uniformly: double-click selection, Ctrl+arrow word movement, Find with whole-word matching, and word-delete commands. [SCI-CHAR]
7. THE language-service crate SHALL be able to provide per-language word-character overrides (e.g., COBOL treating `-` as a word character) that are applied when a file's language is detected or changed. [WB]
8. THE word-character configuration SHALL distinguish between the ASCII range (0x00–0x7F, configured via CharClassify byte table) and the Unicode range (≥U+0080, configured via CharacterCategoryMap word-class rules). [SCI-CHAR]

---

### Requirement 14: Encoding State and Metadata

**User Story:** As a document and status bar component, I want the document's encoding state to be queryable and changeable, so that the status bar can display the current encoding and the user can switch encodings (re-interpreting or converting the file content).

**Source:** [SCI-DOC-11], [FFE], [WB]

#### Acceptance Criteria

1. THE Document SHALL maintain encoding metadata containing: the detected/assigned encoding name (string identifier like "utf-8", "shift-jis", "iso-8859-1"), the encoding family (EncodingFamily enum), the code page number (integer, 0 for UTF-8), and the BOM state (present/absent). [SCI-DOC-11, FFE]
2. WHEN `encoding()` is called on the Document, THE system SHALL return the current encoding metadata. [FFE]
3. WHEN `set_encoding(encoding_name)` is called, THE system SHALL update the encoding metadata. IF the encoding change requires re-interpretation of the buffer content (e.g., reloading the file with a different encoding), THE system SHALL flag that a reload is needed rather than silently reinterpreting bytes. [FFE]
4. WHEN a user requests "Reopen with Encoding", THE system SHALL reload the file from disk using the specified encoding for conversion, discarding the current buffer content. [FFE]
5. THE encoding metadata SHALL be exposed to the status bar UI component for display (e.g., "UTF-8", "UTF-8 with BOM", "Shift-JIS", "ISO-8859-1"). [FFE]
6. WHEN the encoding is changed via "Save As with Encoding", THE system SHALL convert the in-memory UTF-8 content to the target encoding on save without reloading from disk. [FFE]
7. THE system SHALL maintain a registry of supported encoding names and their properties (code page, encoding family, display name, aliases), enabling lookup by name or code page number. [WB]
8. THE encoding registry SHALL support at minimum: UTF-8, UTF-16LE, UTF-16BE, UTF-32LE, UTF-32BE, ISO-8859-1 through ISO-8859-15, Windows-1250 through Windows-1258, Shift-JIS, GBK, EUC-KR, Big5, EUC-JP, EBCDIC (CP037, CP500, CP1047). [WB]

---
