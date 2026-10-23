# One-off: precisely count NON-test production lines in a .rs file, excluding
# every item/module guarded by a #[cfg(test)] attribute (not just the trailing
# test module). A line is "test" if it is a #[cfg(test)] attribute line OR falls
# within the braced block of the item that attribute guards.
# Output: writes the counts to tools/logs/crnr109-render-nontest.log (read back).
param(
    [string]$File = 'c:\workspace\apps\FileForgeWorkbench\.worktrees\crnr109-locale-alias\crates\ff-desktop\src\menu_workspace\render.rs'
)

$lines = Get-Content -LiteralPath $File
$total = $lines.Count
$testLines = 0
$i = 0
while ($i -lt $lines.Count) {
    if ($lines[$i] -match '#\[cfg\(test\)\]') {
        # Count the attribute line itself.
        $testLines++
        $i++
        # Skip any further attribute lines (e.g. doc comments already counted as prod).
        # Find the first line with an opening brace from the guarded item, then
        # balance braces to find the end of that item.
        $depth = 0
        $started = $false
        while ($i -lt $lines.Count) {
            $testLines++
            $open = ([regex]::Matches($lines[$i], '\{')).Count
            $close = ([regex]::Matches($lines[$i], '\}')).Count
            $depth += $open - $close
            if ($open -gt 0) { $started = $true }
            $i++
            if ($started -and $depth -le 0) { break }
            # A cfg(test) on a single-line `use ...;` has no braces: stop after one line.
            if (-not $started -and $lines[$i-1] -match ';\s*$') { break }
        }
    } else {
        $i++
    }
}
$nonTest = $total - $testLines
"File: $File"
"Total lines: $total"
"cfg(test) lines (attributes + guarded blocks): $testLines"
"NON-test production lines: $nonTest"
