# Write the last 40 lines of the Wave 2 gate log to a small file for reading.
Get-Content tools\logs\wave2-gate.txt -Tail 40 | Set-Content tools\logs\wave2-tail.txt
