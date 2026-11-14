<#
.SYNOPSIS
  Run a scoped ff-desktop test with FFWB_UPDATE_GOLDEN=1 so named golden
  fixtures are (re)written, then reviewed by hand before committing.

.DESCRIPTION
  Sets FFWB_UPDATE_GOLDEN for this process only and invokes
  `cargo test -p ff-desktop --bin ffwb <TestFilter>`. Output is written to the
  caller-provided log path (or printed) so the result can be read back by
  effect, per terminal-discipline.md. Overwrites nothing except the golden
  fixtures the test itself rewrites.

.PARAMETER TestFilter
  The cargo test name filter (e.g. a single test function name).

.PARAMETER LogPath
  Optional path to capture combined stdout/stderr.
#>
param(
  [Parameter(Mandatory = $true)] [string] $TestFilter,
  [string] $LogPath
)

$env:FFWB_UPDATE_GOLDEN = '1'
if ($LogPath) {
  cargo test -p ff-desktop --bin ffwb $TestFilter *> $LogPath
} else {
  cargo test -p ff-desktop --bin ffwb $TestFilter
}
exit $LASTEXITCODE
