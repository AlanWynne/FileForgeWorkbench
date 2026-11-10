<#
.SYNOPSIS
  Find processes that reference a given path fragment, to diagnose a locked
  directory (no admin rights required).

.DESCRIPTION
  Lists any process whose ExecutablePath or CommandLine contains the supplied
  path fragment. This covers the two common lock causes: an executable running
  FROM the folder, and a shell/tool launched WITH the folder on its command line.
  It also notes that a process's current working directory (CWD) is NOT visible
  without elevation, so a plain shell cd'd into the folder may not show here.

.PARAMETER Fragment
  A substring of the path to search for (e.g. 'scrm-2c-capture-lifecycle').

.EXAMPLE
  pwsh -NoProfile -File tools\powershell\find-path-holder.ps1 -Fragment 'scrm-2c-capture-lifecycle'
#>
param(
    [Parameter(Mandatory = $true)]
    [string] $Fragment
)

Write-Output "Searching running processes for path fragment: $Fragment"
Write-Output ("-" * 60)

$matches = Get-CimInstance Win32_Process | Where-Object {
    ($_.CommandLine -and $_.CommandLine -like "*$Fragment*") -or
    ($_.ExecutablePath -and $_.ExecutablePath -like "*$Fragment*")
}

if (-not $matches) {
    Write-Output "No process has that fragment in its ExecutablePath or CommandLine."
    Write-Output ""
    Write-Output "NOTE: a process's current working directory (CWD) is NOT checked"
    Write-Output "here (it needs elevation to read). If a shell or editor is simply"
    Write-Output "sitting INSIDE the folder, close/cd it out. Otherwise the holder is"
    Write-Output "likely an indexer/antivirus/Explorer preview touching the folder."
} else {
    foreach ($p in $matches) {
        Write-Output "PID : $($p.ProcessId)"
        Write-Output "Name: $($p.Name)"
        Write-Output "Exe : $($p.ExecutablePath)"
        Write-Output "Cmd : $($p.CommandLine)"
        Write-Output ("-" * 60)
    }
}
