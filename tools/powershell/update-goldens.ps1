<#
.SYNOPSIS
  Regenerate ff-desktop golden text fixtures for a given test filter.

.DESCRIPTION
  Sets FFWB_UPDATE_GOLDEN=1 for the cargo child process and runs the scoped
  golden test(s) matching the supplied filter, so the fixtures under
  crates/ff-desktop/src/shell/golden/ are (re)written. Review the regenerated
  fixtures by hand (git diff) before relying on them.

  This exists because the execution proxy splits ';'/newline command chains, so
  an inline "set env then cargo" one-liner cannot reliably set the variable for
  the cargo child. Running it from a script sets it in-process.

.PARAMETER Filter
  The cargo test name filter (substring), e.g. menu_kind_pom_markdown_matches_golden.
  Defaults to running the POM/SCRM goldens that change under CR-CH-116.

.OUTPUTS
  cargo output goes to the console (captured by the execution proxy result file).
#>
param(
    [string]$Filter = ""
)

$env:FFWB_UPDATE_GOLDEN = "1"

$filters = if ($Filter) {
    @($Filter)
} else {
    @(
        "menu_kind_pom_markdown_matches_golden",
        "menu_kind_scrm_box_and_markdown_match_golden",
        "menu_kind_pom_snapshot_matches_golden_pfshow_on_with_message"
    )
}

foreach ($f in $filters) {
    Write-Output "=== updating golden for filter: $f ==="
    cargo test -p ff-desktop $f
}
