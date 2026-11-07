$ErrorActionPreference = 'Stop'
$artTaskRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
foreach ($artTaskPort in @(7297,7298)) {
    $artTaskConnections = @(Get-NetTCPConnection -LocalPort $artTaskPort -State Listen -ErrorAction SilentlyContinue)
    foreach ($artTaskProcessId in @($artTaskConnections.OwningProcess | Select-Object -Unique)) {
        $artTaskProcess = Get-CimInstance Win32_Process -Filter "ProcessId=$artTaskProcessId"
        $artTaskExpected = if ($artTaskPort -eq 7297) {'serve-ranked-body-review.cjs'} else {'editor/server.js'}
        if ($artTaskProcess.Name -ne 'node.exe' -or -not $artTaskProcess.CommandLine.Contains($artTaskExpected)) {throw "Port $artTaskPort belongs to another application; leaving it untouched"}
        if ($artTaskPort -eq 7298 -and -not $artTaskProcess.CommandLine.Contains((Join-Path $artTaskRoot 'backups/ranked-body-editor'))) {throw 'Editor fixture identity mismatch; leaving it untouched'}
        Stop-Process -Id $artTaskProcessId
    }
}
$artTaskRtk = (Get-Command rtk.exe).Source
$artTaskLogs = Join-Path $artTaskRoot 'backups'
Start-Process -FilePath $artTaskRtk -ArgumentList @('proxy','node','tools/serve-ranked-body-review.cjs','7297') -WorkingDirectory $artTaskRoot -WindowStyle Hidden -RedirectStandardOutput (Join-Path $artTaskLogs 'ranked-review-server.log') -RedirectStandardError (Join-Path $artTaskLogs 'ranked-review-server-error.log')
Start-Process -FilePath $artTaskRtk -ArgumentList @('proxy','python','tools/serve-ranked-editor.py','7298') -WorkingDirectory $artTaskRoot -WindowStyle Hidden -RedirectStandardOutput (Join-Path $artTaskLogs 'ranked-editor-server.log') -RedirectStandardError (Join-Path $artTaskLogs 'ranked-editor-server-error.log')
Write-Output 'Restarted verified local art-preview servers only; game and other ports untouched.'
