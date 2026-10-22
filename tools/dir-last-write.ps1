Get-ChildItem -Path "C:\workspace\apps\FileForgeWorkbench" -File -Recurse |
    Sort-Object LastWriteTime |
    Select-Object LastWriteTime, Length, FullName |
    Out-File -FilePath "C:\workspace\apps\FileForgeWorkbench\tools\logs\dir-last-write.log" -Encoding utf8
