param()
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskServer = Join-Path $taskRoot 'editor/server.js'
$taskLogDir = Join-Path $taskRoot 'backups/akane-review-server'
New-Item -ItemType Directory -Path $taskLogDir -Force | Out-Null
$taskListener = Get-NetTCPConnection -LocalPort 7304 -State Listen -ErrorAction SilentlyContinue | Select-Object -First 1
if ($taskListener) {
    $taskProcess = Get-CimInstance Win32_Process -Filter "ProcessId=$($taskListener.OwningProcess)"
    if ($taskProcess.Name -ne 'node.exe' -or $taskProcess.CommandLine -notlike "*$taskServer*--port*7304*") {
        throw 'Port 7304 belongs to a different process; it was left running.'
    }
    Write-Output 'Akane preview is already running: http://localhost:7304/akane-review.html'
    exit 0
}
$taskNode = (Get-Command node).Source
$taskChild = Start-Process -FilePath $taskNode -ArgumentList @($taskServer, '--port', '7304', '--no-open') -WorkingDirectory $taskRoot -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $taskLogDir 'stdout.log') -RedirectStandardError (Join-Path $taskLogDir 'stderr.log')
Write-Output "Preview PID $($taskChild.Id): http://localhost:7304/akane-review.html"
