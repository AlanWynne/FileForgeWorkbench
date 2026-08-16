# Requirements Document

## Introduction

This feature specifies the **Language Service** for FileForgeWorkbench — the `ff-language-service` crate. The language service is the foundational layer responsible for language detection, language definition management, multi-line lexer state persistence, and content-based language identification. It provides the structural metadata and detection logic that the syntax-highlighting engine consumes to tokenize and colour source code.

The language service is **GUI-independent** — it has no rendering dependency. It loads language definitions from TOML files, matches files to languages via extension or content inspection, manages per-line lexer state for multi-line constructs (strings, comments, heredocs), and exposes a plugin-extensible registration model for adding new language definitions at runtime.

This specification merges requirements from three primary sources:

- **FileForgeEditor MVP Requirement 6**: LanguageService loads all `*.toml` from `languages/` directory, registers LanguageDefinition instances, detects language by file extension, LexicalHighlighter produces HighlightSpan for keyword occurrences, keywords rendered in distinct colour, `line_comment` marks comment spans.
- **Lexilla concepts (adapted to Rust)**: Lexer registration infrastructure, state-based scanning with multi-line state persistence per line, content-based detection (shebang, magic bytes, first-line patterns), multiple language definitions coexisting, sub-languages within a document (embedded languages), keyword lists per language (up to 9 keyword sets), property-based configuration per lexer.
- **Workbench Architecture**: GUI-independent language detection and definition loading, plugin-extensible language registration, TOML-based language definition format, integration with the syntax-highlighting engine.

The `ff-language-service` crate is a Wave 7 (Language and Highlighting) component. It depends on `ff-logging` for diagnostics, `ff-config` (configuration-system) for language-related settings, and is consumed by `ff-syntax-highlighting` for token production, `ff-auto-indentation` for language-aware indent rules, and `ff-plugin` (plugin-architecture) for runtime language registration.

**Source references:**
- **[FFE-MVP-6]** = FileForgeEditor mvp-implementation Requirement 6: Syntax Highlighting
- **[LEX]** = Lexilla lexer infrastructure, lexer support, and lexer utilities (adapted to Rust)
- **[WB]** = Workbench Platform Architecture Brief

## Glossary

- **LanguageService**: The central registry that owns all loaded LanguageDefinition instances, resolves file-to-language mappings, and provides language metadata to the syntax-highlighting engine. [FFE-MVP-6, WB]
- **LanguageDefinition**: A TOML-backed struct describing a single language's metadata: name, file extensions, keywords, comment syntax, string delimiters, multi-line constructs, and property overrides. [FFE-MVP-6, LEX]
- **LanguageId**: A unique identifier (lowercase ASCII string) for a registered language (e.g., `"rust"`, `"cobol"`, `"jcl"`). Used as the key in the language registry. [WB]
- **KeywordSet**: A sorted list of keywords belonging to a numbered category (0–8) within a language definition. Languages may define up to 9 keyword sets for different syntactic roles (keywords, types, builtins, etc.). [LEX]
- **LexerState**: An integer value representing the lexer's internal state at the end of a line. Persisted per line to enable incremental re-highlighting from any point in the document. [LEX]
- **ContentDetector**: A function or rule that inspects the first bytes or first line of a file to determine its language when extension matching is ambiguous or absent. [LEX]
- **ShebangPattern**: A regex or prefix pattern matching the `#!` line at the start of a script file to identify the interpreter and thus the language. [LEX]
- **MagicBytes**: A fixed byte sequence at offset 0 of a file that identifies its format (e.g., ELF header, PDF header). [LEX]
- **EmbeddedLanguage**: A sub-language region within a document (e.g., JavaScript inside HTML `<script>` tags, SQL inside a Rust string literal) that requires a different language definition for that span. [LEX]
- **LanguageProperty**: A key-value configuration pair scoped to a specific language definition, used to parameterize lexer behaviour (e.g., `fold.comment=1`, `lexer.sql.numbering=2`). [LEX]
- **LineComment**: A character or string that marks the beginning of a single-line comment extending to end-of-line. [FFE-MVP-6]
- **BlockComment**: A pair of opening and closing delimiters that mark a multi-line comment region. [LEX]
- **TOML Definition File**: A `.toml` file in the `languages/` directory conforming to the language definition schema. [FFE-MVP-6, WB]

---

## Requirements

### Requirement 1: Language Definition Loading from TOML

**User Story:** As a workbench user, I want language definitions loaded automatically from TOML files at startup, so that syntax highlighting is available for all configured languages without manual setup.

**Source:** [FFE-MVP-6] AC 1, [WB] TOML-based configuration. Cross-references: `configuration-system` (languages/ directory), `syntax-highlighting` (consumes definitions).

#### Acceptance Criteria

1. WHEN the LanguageService initializes, THE system SHALL scan all `*.toml` files in the `languages/` directory (as configured by the configuration-system) and parse each file as a LanguageDefinition. [FFE-MVP-6]
2. EACH TOML language definition file SHALL conform to a well-defined schema containing at minimum: `name` (string), `language_id` (string), `extensions` (array of strings), and `keywords` (table mapping set numbers "0"–"8" to arrays of keyword strings). [FFE-MVP-6, LEX]
3. THE language definition schema SHALL support the following optional fields: `line_comment` (string), `block_comment_start` (string), `block_comment_end` (string), `string_delimiters` (array of strings), `character_delimiter` (string), `escape_character` (string), `fold_keywords` (table), `properties` (table of key-value pairs), and `embedded_languages` (array of embedded language descriptors). [FFE-MVP-6, LEX]
4. WHEN a TOML definition file contains syntax errors or fails schema validation, THE LanguageService SHALL skip that file, emit a WARN-level log record identifying the file path and parse error, and continue loading remaining definition files. [WB]
5. WHEN multiple definition files declare the same `language_id`, THE LanguageService SHALL use the first loaded definition, emit a WARN-level log record identifying the duplicate, and discard subsequent definitions with the same identifier. [LEX]
6. THE LanguageService SHALL support loading definitions from multiple directories in a defined search order: built-in defaults directory → user configuration directory → project-local directory, with later directories able to override earlier definitions for the same `language_id`. [WB]
7. AFTER all definitions are loaded, THE LanguageService SHALL emit a DEBUG-level log record listing the count of successfully loaded language definitions and the count of any skipped files. [WB]

---

### Requirement 2: Language Detection by File Extension

**User Story:** As a workbench user, I want the editor to automatically detect my file's language based on its extension, so that syntax highlighting activates without manual language selection.

**Source:** [FFE-MVP-6] AC 2, AC 5. Cross-references: `document-model` (file path metadata), `syntax-highlighting` (uses detected language).

#### Acceptance Criteria

1. WHEN a file is opened, THE LanguageService SHALL detect the language by matching the file's extension (case-insensitive) against the `extensions` array in each loaded LanguageDefinition. [FFE-MVP-6]
2. THE extension matching SHALL be case-insensitive: a definition with `extensions = ["rs"]` SHALL match files named `main.RS`, `main.Rs`, and `main.rs`. [WB]
3. THE extension matching SHALL support compound extensions: a definition with `extensions = ["test.ts"]` SHALL match `foo.test.ts` with higher priority than a definition matching only `.ts`. [LEX]
4. WHEN exactly one LanguageDefinition matches the file extension, THE LanguageService SHALL return that definition's `language_id` as the detected language. [FFE-MVP-6]
5. WHEN no LanguageDefinition matches the file extension, THE LanguageService SHALL return a "plain text" language identifier indicating no syntax highlighting applies. [FFE-MVP-6]
6. WHEN multiple LanguageDefinitions match the same file extension, THE LanguageService SHALL use the definition with the highest priority value (defined in the TOML file via an optional `priority` field, defaulting to 0), falling back to alphabetical order of `language_id` if priorities are equal. [LEX]
7. THE LanguageService SHALL provide a method to manually override the detected language for a document, allowing users to force a specific language regardless of extension. [WB]

---

### Requirement 3: Content-Based Language Detection

**User Story:** As a workbench user, I want files without extensions (or with ambiguous extensions) to be identified by their content, so that scripts with shebangs and special files are highlighted correctly.

**Source:** [LEX] content-based detection. Cross-references: `document-model` (first-line access), `file-operations` (file opening flow).

#### Acceptance Criteria

1. WHEN extension-based detection returns "plain text" (no match), THE LanguageService SHALL attempt content-based detection by inspecting the first line and first bytes of the file. [LEX]
2. THE LanguageService SHALL support shebang detection: IF the first line of a file begins with `#!`, THEN the LanguageService SHALL extract the interpreter name and match it against `shebang_patterns` defined in language definitions (e.g., `shebang_patterns = ["python", "python3"]` matches `#!/usr/bin/env python3`). [LEX]
3. THE LanguageService SHALL support magic-byte detection: IF the first N bytes of a file match a `magic_bytes` pattern defined in a language definition (e.g., `magic_bytes = [0x7F, 0x45, 0x4C, 0x46]` for ELF), THEN the LanguageService SHALL identify the file as that language. [LEX]
4. THE LanguageService SHALL support first-line pattern detection: IF the first line of a file matches a `first_line_pattern` regex defined in a language definition (e.g., `first_line_pattern = "^<\\?xml"` for XML), THEN the LanguageService SHALL identify the file as that language. [LEX]
5. THE content-based detection SHALL apply detection rules in priority order: magic bytes (highest) → shebang → first-line pattern (lowest). The first successful match wins. [LEX]
6. WHEN content-based detection also fails to identify a language, THE LanguageService SHALL retain the "plain text" classification. [LEX]
7. THE LanguageService SHALL perform content-based detection using only the first 8192 bytes of the file, to avoid scanning large files entirely for detection purposes. [LEX]

---

### Requirement 4: Multi-Line Lexer State Persistence

**User Story:** As a workbench developer, I want the language service to track lexer state at the end of each line, so that the syntax-highlighting engine can incrementally re-highlight from any changed line without re-scanning the entire file.

**Source:** [LEX] state-based scanning, per-line state. Cross-references: `syntax-highlighting` (consumes per-line state), `document-model` (line change notifications).

#### Acceptance Criteria

1. THE LanguageService SHALL maintain a per-line state vector that stores the LexerState (integer) at the end of each document line. [LEX]
2. WHEN a line is highlighted, THE syntax-highlighting engine SHALL provide the resulting end-of-line LexerState back to the LanguageService, which SHALL store it in the per-line state vector at the corresponding line index. [LEX]
3. WHEN a line is modified, THE LanguageService SHALL mark that line's stored state as invalid, signalling that re-highlighting is needed from that line forward. [LEX]
4. WHEN re-highlighting begins at a line N, THE LanguageService SHALL provide the stored LexerState from line N-1 (or the initial state 0 if N is line 0) as the starting state for the lexer. [LEX]
5. WHEN re-highlighting produces the same end-of-line LexerState for a line as was previously stored, THE LanguageService SHALL stop propagating re-highlighting to subsequent lines (incremental highlighting termination condition). [LEX]
6. WHEN lines are inserted into the document, THE LanguageService SHALL insert corresponding entries into the per-line state vector with an invalid/uninitialized state marker. [LEX]
7. WHEN lines are deleted from the document, THE LanguageService SHALL remove the corresponding entries from the per-line state vector and invalidate the state of the line following the deletion point. [LEX]
8. THE per-line state vector SHALL use a compact representation (e.g., `Vec<i32>`) and SHALL support documents with millions of lines without excessive memory overhead. [LEX]

---

### Requirement 5: Keyword Lists

**User Story:** As a workbench user, I want language definitions to support multiple keyword categories, so that different syntactic elements (keywords, types, builtins, constants) are highlighted distinctly.

**Source:** [FFE-MVP-6] AC 3, [LEX] keyword lists (up to 9 sets). Cross-references: `syntax-highlighting` (keyword matching logic), `theme-and-appearance` (maps keyword sets to colours).

#### Acceptance Criteria

1. EACH LanguageDefinition SHALL support up to 9 keyword sets, numbered 0 through 8, where each set contains a sorted list of keyword strings. [LEX]
2. THE keyword sets SHALL be defined in the TOML definition file under a `[keywords]` table with string keys `"0"` through `"8"`, each mapping to an array of keyword strings. [LEX]
3. WHEN a keyword set is loaded, THE LanguageService SHALL sort the keywords alphabetically and build a first-character index for O(1) lookup of the starting position for each initial character. [LEX]
4. THE LanguageService SHALL provide an `in_keyword_set(word, set_number) -> bool` method that performs case-sensitive membership testing against the specified keyword set. [LEX]
5. THE LanguageService SHALL provide an `in_keyword_set_case_insensitive(word, set_number) -> bool` method that performs case-insensitive membership testing by comparing lowercased input against pre-lowercased keywords. [LEX]
6. WHEN a language definition specifies `case_sensitive_keywords = false`, THE LanguageService SHALL store keywords in lowercase and use case-insensitive matching for all keyword lookups in that language. [LEX]
7. EACH keyword set number SHALL map to a distinct style identifier that the theme system can colour independently (e.g., set 0 = "keyword", set 1 = "type", set 2 = "builtin"). The mapping from set number to semantic name SHALL be configurable in the language definition. [LEX]

---

### Requirement 6: Comment and String Syntax Definitions

**User Story:** As a workbench user, I want my language's comments and strings to be recognized and highlighted correctly, including multi-line constructs, so that code readability is maximized.

**Source:** [FFE-MVP-6] AC 6 (line_comment), [LEX] block comments, string delimiters. Cross-references: `syntax-highlighting` (uses these definitions for span production).

#### Acceptance Criteria

1. WHERE a LanguageDefinition contains a `line_comment` value, THE LanguageService SHALL expose this to the highlighting engine so that content from the comment marker to end-of-line is treated as a comment span. [FFE-MVP-6]
2. WHERE a LanguageDefinition contains `block_comment_start` and `block_comment_end` values, THE LanguageService SHALL expose these delimiters so that multi-line comment regions are recognized across line boundaries. [LEX]
3. THE LanguageService SHALL support multiple line-comment styles per language: the `line_comment` field MAY be a string (single style) or an array of strings (multiple styles, e.g., `//` and `///` for Rust). [LEX]
4. WHERE a LanguageDefinition contains `string_delimiters`, THE LanguageService SHALL expose the list of string-opening characters or sequences (e.g., `["\"", "'", "`"]`) so that the highlighting engine can identify string literal spans. [LEX]
5. WHERE a LanguageDefinition specifies an `escape_character` (e.g., `\`), THE LanguageService SHALL expose this so that the highlighting engine correctly handles escaped delimiters within strings (e.g., `\"` does not end a string). [LEX]
6. THE LanguageService SHALL support language definitions that define heredoc-style multi-line strings via a `heredoc_patterns` field (array of regex patterns matching heredoc start markers), enabling state-based tracking of heredoc boundaries. [LEX]
7. THE LanguageService SHALL expose all comment and string syntax metadata as read-only accessors on the LanguageDefinition struct, allowing the highlighting engine to query them without mutating language service state. [WB]

---

### Requirement 7: Embedded Languages (Sub-Languages)

**User Story:** As a workbench user editing HTML files, I want embedded JavaScript and CSS to be highlighted using their own language rules, so that mixed-language documents are readable.

**Source:** [LEX] sub-languages within a document. Cross-references: `syntax-highlighting` (switches lexer context), `document-model` (range-based language assignment).

#### Acceptance Criteria

1. THE LanguageDefinition schema SHALL support an `embedded_languages` field: an array of embedded language descriptors, each specifying a `language_id`, `start_pattern` (regex or string matching the embedding start marker), and `end_pattern` (regex or string matching the embedding end marker). [LEX]
2. WHEN the syntax-highlighting engine encounters an embedded language start pattern during tokenization, THE LanguageService SHALL provide the corresponding embedded LanguageDefinition so the highlighter can switch to the embedded language's keyword sets and rules. [LEX]
3. WHEN the syntax-highlighting engine encounters an embedded language end pattern, THE LanguageService SHALL signal a return to the host language's rules and state. [LEX]
4. THE LanguageService SHALL support nesting of embedded languages to at least 3 levels (e.g., HTML containing JavaScript containing a template literal with embedded HTML). [LEX]
5. THE per-line state for lines containing embedded language transitions SHALL encode both the host language state and the embedded language identity, enabling correct incremental re-highlighting. [LEX]
6. IF an embedded language descriptor references a `language_id` that is not registered, THEN THE LanguageService SHALL emit a WARN-level log record and treat the embedded region as unstyled text. [WB]

---

### Requirement 8: Language Property Configuration

**User Story:** As a workbench user, I want to configure per-language lexer behaviour through properties, so that I can customize how individual languages are highlighted without modifying the definition file.

**Source:** [LEX] property-based configuration per lexer. Cross-references: `configuration-system` (language profiles in `languages/` subdirectory).

#### Acceptance Criteria

1. EACH LanguageDefinition SHALL support an optional `[properties]` table containing key-value pairs (string keys, string values) that parameterize lexer behaviour for that language. [LEX]
2. THE LanguageService SHALL provide a `get_property(language_id, key) -> Option<String>` method that retrieves a property value for a specific language, first checking user/project configuration overrides and then falling back to the definition's built-in properties. [LEX]
3. THE LanguageService SHALL provide a `get_property_int(language_id, key, default) -> i64` convenience method that parses the property value as an integer, returning `default` if the key is absent or unparseable. [LEX]
4. THE LanguageService SHALL provide a `get_property_bool(language_id, key, default) -> bool` convenience method that parses the property value as a boolean (`"1"`, `"true"`, `"yes"` → true; `"0"`, `"false"`, `"no"` → false), returning `default` if absent or unparseable. [LEX]
5. WHEN the configuration-system hot-reloads a language profile file in the `languages/` subdirectory, THE LanguageService SHALL update the affected language's property values and notify the syntax-highlighting engine that re-highlighting is needed for documents using that language. [WB]
6. LANGUAGE properties SHALL be overridable by user and project configuration layers using the key path `languages.{language_id}.{property_key}`, following the standard configuration-system layered model. [WB]

---

### Requirement 9: Plugin-Extensible Language Registration

**User Story:** As a plugin developer, I want to register new language definitions at runtime through the plugin API, so that my plugin can add support for custom or proprietary languages without modifying built-in definition files.

**Source:** [WB] plugin-extensible language definitions. Cross-references: `plugin-architecture` (PluginContext capability registration), `syntax-highlighting` (uses registered languages).

#### Acceptance Criteria

1. THE LanguageService SHALL provide a registration API that accepts a LanguageDefinition struct and adds it to the active registry, making it immediately available for language detection and highlighting. [WB]
2. WHEN a plugin registers a LanguageDefinition via PluginContext, THE LanguageService SHALL validate the definition against the schema (required fields present, keyword sets well-formed) before accepting it. [WB]
3. IF a plugin attempts to register a LanguageDefinition with a `language_id` that already exists, THE LanguageService SHALL reject the registration and return an error — plugins cannot override built-in or previously registered definitions. [WB]
4. WHEN a plugin that registered a language is unloaded (shutdown lifecycle phase), THE LanguageService SHALL remove that plugin's language definitions from the registry and emit a DEBUG-level log record indicating which languages were deregistered. [WB]
5. THE registration API SHALL accept the same schema as TOML-loaded definitions (keyword sets, comment syntax, extension mappings, properties, embedded languages), ensuring feature parity between file-loaded and plugin-registered definitions. [WB]
6. PLUGINS SHALL register language definitions by advertising the `LanguageSupport` capability through the plugin-architecture's Capability_Registry. [WB]
7. WHEN a plugin-registered language is deregistered, THE LanguageService SHALL notify the syntax-highlighting engine to fall back to "plain text" for any documents that were using the removed language. [WB]

---

### Requirement 10: Language Service Query API

**User Story:** As a workbench developer, I want a clean query API to enumerate available languages, retrieve definitions, and resolve language-specific metadata, so that UI components (language picker, status bar) and subsystems (auto-indentation, macro engine) can access language information uniformly.

**Source:** [WB] GUI-independent API surface. Cross-references: `menu-and-statusbar` (language display), `auto-indentation` (indent rules lookup), `document-model` (language assignment per document).

#### Acceptance Criteria

1. THE LanguageService SHALL provide a `list_languages() -> Vec<LanguageSummary>` method that returns a list of all registered languages with their `language_id`, display name, and file extension list. [WB]
2. THE LanguageService SHALL provide a `get_definition(language_id) -> Option<&LanguageDefinition>` method that returns an immutable reference to the full definition for a given language identifier. [WB]
3. THE LanguageService SHALL provide a `detect_language(file_path, first_line, first_bytes) -> LanguageId` method that performs the full detection pipeline (extension → content-based) and returns the resolved language identifier. [FFE-MVP-6, LEX]
4. THE LanguageService SHALL provide a `extensions_for(language_id) -> &[String]` method that returns the list of file extensions associated with a language. [WB]
5. THE LanguageService SHALL provide a `language_for_extension(extension) -> Option<LanguageId>` method that performs extension-only lookup without content-based fallback. [FFE-MVP-6]
6. ALL query methods on the LanguageService SHALL be callable from any thread without requiring mutable access — the public query API SHALL use interior immutability (`&self`) backed by appropriate synchronization. [WB]
7. THE LanguageService SHALL be constructable and testable without any GUI framework, filesystem, or running application — accepting a list of LanguageDefinition structs or a directory path for unit testing. [WB]
