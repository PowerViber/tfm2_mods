param([switch]$Restart)
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskServer = Join-Path $taskRoot 'editor/server.js'
$taskLogDir = Join-Path $taskRoot 'backups/kagami-review-server'
New-Item -ItemType Directory -Path $taskLogDir -Force | Out-Null
$taskListener = Get-NetTCPConnection -LocalPort 7307 -State Listen -ErrorAction SilentlyContinue | Select-Object -First 1
if ($taskListener) {
    $taskProcess = Get-CimInstance Win32_Process -Filter "ProcessId=$($taskListener.OwningProcess)"
    if ($taskProcess.Name -ne 'node.exe' -or $taskProcess.CommandLine -notlike "*$taskServer*--port*7307*") {
        throw 'Port 7307 belongs to a different process; it was left running.'
    }
    if(-not $Restart){Write-Output 'Preview is already running: http://localhost:7307/kagami-review.html';exit 0}
    Stop-Process -Id $taskProcess.ProcessId
}
$taskNode = (Get-Command node).Source
$taskChild = Start-Process -FilePath $taskNode -ArgumentList @($taskServer, '--port', '7307', '--no-open') -WorkingDirectory $taskRoot -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $taskLogDir 'stdout.log') -RedirectStandardError (Join-Path $taskLogDir 'stderr.log')
Write-Output "Preview PID $($taskChild.Id): http://localhost:7307/kagami-review.html"
