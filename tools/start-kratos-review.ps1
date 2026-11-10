param([switch]$Restart)
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskServer = Join-Path $taskRoot 'editor/server.js'
$taskLogDir = Join-Path $taskRoot 'backups/kratos-review-server'
New-Item -ItemType Directory -Path $taskLogDir -Force | Out-Null
$taskListener = Get-NetTCPConnection -LocalPort 7302 -State Listen -ErrorAction SilentlyContinue | Select-Object -First 1
if ($taskListener) {
    if (-not $Restart) { Write-Output 'Preview already listening: http://localhost:7302/kratos-review.html'; exit 0 }
    $taskProcess = Get-CimInstance Win32_Process -Filter "ProcessId=$($taskListener.OwningProcess)"
    if ($taskProcess.Name -ne 'node.exe' -or $taskProcess.CommandLine -notmatch 'server\.js.*--port\s+7302') {
        throw 'Port 7302 belongs to a different process; it was left running.'
    }
    Stop-Process -Id $taskProcess.ProcessId
}
$taskNode = (Get-Command node).Source
$taskChild = Start-Process -FilePath $taskNode -ArgumentList @($taskServer, '--port', '7302', '--no-open') -WorkingDirectory $taskRoot -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $taskLogDir 'stdout.log') -RedirectStandardError (Join-Path $taskLogDir 'stderr.log')
Write-Output "Preview PID $($taskChild.Id): http://localhost:7302/kratos-review.html"
