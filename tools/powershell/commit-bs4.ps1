# commit-bs4.ps1 -- Stage and commit Phase BS.4 KSDS alternate index support.
# Usage: .\tools\powershell\commit-bs4.ps1
# Run from the repository root.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$files = @(
    'crates/ff-dscatalog/src/codecs/fixed.rs',
    'crates/ff-dscatalog/src/codecs/text.rs',
    'crates/ff-dscatalog/src/codecs/variable.rs',
    'crates/ff-dscatalog/src/storage/mod.rs',
    'crates/ff-dscatalog/src/storage/native.rs',
    'crates/ff-dscatalog/src/storage/rrds.rs',
    'crates/ff-dscatalog/src/storage/sqlite_record.rs',
    'crates/ff-dscatalog/src/storage/esds.rs',
    'docs/quality/TCR.md',
    'docs/specs/dataset-catalog/design.md',
    'docs/specs/dataset-catalog/requirements.md',
    'docs/specs/dataset-catalog/tasks.md',
    'docs/specs/project-master/tasks.md'
)

foreach ($f in $files) {
    git add $f
}

$message = @'
feat(ff-dscatalog): Phase BS.4 -- KSDS alternate index support

- Add AlternateIndex struct and ALT_INDEX_REGISTRY constant
- Create KSDS_ALT_INDEXES registry table in initialise_database
- Implement add_alternate_index(), rebuild_alternate_index(),
  lookup_by_alternate_key(), list_alternate_indexes()
- Generalise extract_key_field() to accept offset/length directly
- Fix pre-existing clippy manual_is_multiple_of lints in fixed.rs
  and text.rs
- Add 4 new tests (170 total in ff-dscatalog, 0 failures)
- Update TCR.md Req 21.5 to PASS
- Mark tasks 19.4 and BS.4 complete in tasks.md files
'@

git commit -m $message
