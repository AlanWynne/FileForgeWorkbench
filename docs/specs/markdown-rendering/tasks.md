# Implementation Tasks

Tasks for CR-NR-101 (markdown-rendering). All work is TDD-first: write the failing test, confirm red, implement, confirm green. Every test carries `// Validates: Requirement X.Y`. Scoped checks only (`cargo test -p ff-md-style`, etc.); the full gate is the owner's manual step.

- [ ] 1. Scaffold the `ff-md-style` crate
  - [ ] 1.1 Create `crates/ff-md-style` with Cargo.toml (deps: ff-config, ff-theme, ff-logging, serde, toml; optional `egui` feature adding egui dep) and add it to the workspace members
  - [ ] 1.2 Add `lib.rs` thin coordinator with crate docs and module declarations

- [ ] 2. Config schema and built-in default (Req 1.5, 2.1, 2.2, 3.1, 4.1, 5.1, 6.1, 7.1, 8.6)
  - [ ] 2.1 Write `config.rs` structs (MarkdownStyleConfig + element sub-structs) with serde derives and Debug
  - [ ] 2.2 Write `defaults.rs` built-in default (monotonic heading sizes H1>...>H6, Req 2.4; sensible layout tokens) with tests asserting the default is well-formed
  - [ ] 2.3 Test: default heading sizes decrease monotonically H1..H6 (Req 2.4)

- [ ] 3. Colour token references (Req 8.1-8.6)
  - [ ] 3.1 Write `colour_ref.rs` ColourTokenRef newtype + parse/validate
  - [ ] 3.2 Implement resolve against ThemePalette (editor/syntax/ui groups) returning ColourRGBA
  - [ ] 3.3 Test: known tokens resolve to palette values; unknown token logs warning + falls back (editor.foreground / editor.background) (Req 8.4)
  - [ ] 3.4 Test: config stores no literal colours -- every colour field is a ColourTokenRef (Req 8.1, 8.5)

- [ ] 4. Loader with partial-definition merge and validation (Req 1.1-1.7)
  - [ ] 4.1 Write `loader.rs`: load TOML via ff-config, merge present values over the default (Req 1.5)
  - [ ] 4.2 Per-value validation + clamp + warning (font sizes 6-72 Req 2.5/3.6; negative spacing Req 4.4/6.4; width bounds Req 7.5)
  - [ ] 4.3 Invalid TOML retains previous/default + warning (Req 1.3); invalid individual values use per-token default + warning (Req 1.4)
  - [ ] 4.4 Round-trip: parse(serialise(config)) == config (Req 1.7) -- property test
  - [ ] 4.5 Define the `markdown.style_file` config key constant + default (empty) for consumers to register

- [ ] 5. CSS emitter (Req 9.2, 9.6, and all element Reqs for HTML fidelity)
  - [ ] 5.1 Write `css.rs` emit_css(config, palette) -> String targeting pulldown-cmark output selectors (h1-h6, p, code, pre, blockquote, ul/ol/li, table/th/td, hr, a, del, input[type=checkbox], footnote)
  - [ ] 5.2 Headings (Req 2), body/inline/link (Req 3), lists/blockquote/tasklist (Req 4), code blocks + token classes (Req 5), tables/zebra/rule (Req 6), layout max-width + centring (Req 7)
  - [ ] 5.3 Test: emitted CSS is self-contained (no external url()/@import) (Req 9.6)
  - [ ] 5.4 Test: resolved colours appear as #RRGGBB(AA) from the palette; max_content_width maps to max-width+margin auto; width 0 omits constraint (Req 7.2, 7.4)

- [ ] 6. ff-html-export integration (Req 9.2)
  - [ ] 6.1 Add `build_standalone_html_with_css(title, body, css)` to ff-html-export; keep `build_standalone_html` delegating to it with default-config CSS (backward compatible)
  - [ ] 6.2 Test: existing build_standalone_html output still contains the expected elements; the _with_css variant injects the supplied stylesheet

- [ ] 7. egui style mapping (feature `egui`) (Req 9.1, 9.4, 9.5)
  - [ ] 7.1 Write `egui_map.rs` EguiStyleMapping::apply(config, palette, &mut Style) -- heading TextStyle sizes, body override_text_color
  - [ ] 7.2 configure_viewer(config, CommonMarkViewer) -> default_width (max content width), indentation_spaces (list indent), syntax_theme_dark/light (code highlighting + VisualMode), max_image_width
  - [ ] 7.3 Code-highlighting enable/disable selects syntect theme vs plain monospace (Req 5.3-5.5)
  - [ ] 7.4 egui_kittest build_ui test: mapping applied -> assert max width set, heading text styles present, syntax theme selected (model-level, not pixels)

- [ ] 8. Hot-reload and change notification (Req 10.1-10.5)
  - [ ] 8.1 Write MarkdownStyleHandle (Arc<RwLock<Arc<MarkdownStyleConfig>>> + epoch) with atomic swap
  - [ ] 8.2 ff-config hot-reload callback on markdown.style_file reloads + swaps (Req 10.1, 10.2)
  - [ ] 8.3 Re-resolution on theme palette-change is implicit (colours are refs) -- test that a VisualMode change changes resolved CSS/egui colours with no config reload (Req 10.3)
  - [ ] 8.4 Change-notification epoch bump on reload (Req 10.4); invalid reload retains previous (Req 10.5)

- [ ] 9. Export-defaults and discoverability (Req 11.1-11.5)
  - [ ] 9.1 Serialiser writes the effective config to TOML with per-section comments (Req 11.3)
  - [ ] 9.2 Test: built-in default references theme tokens for all colours (Req 11.2); default not written to disk automatically (Req 11.5) -- asserted by the consumer, documented here

- [ ] 10. Both-paths-driven conformance test (Req 9.3)
  - [ ] 10.1 Test enumerating element classes; assert each is addressed by both emit_css output and the egui mapping (no element styled by only one path)

- [ ] 11. Update TCR rows for every markdown-rendering criterion to PASS as covered; document any MANUAL (pixel-exact appearance) with a reason
