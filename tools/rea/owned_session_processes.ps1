param([switch]$Stop)
$ErrorActionPreference = 'Stop'
$taskProcesses = @(Get-CimInstance Win32_Process)
$taskRoots = @($taskProcesses | Where-Object {
    $_.Name -eq 'python.exe' -and $_.CommandLine -like '*tools/rea/rea_session.py --queue C:/Users/arian/.cache/tfm2-rea/session-contender*'
})
if ($taskRoots.Count -ne 1) { throw "Expected exactly one owned investigation runner; found $($taskRoots.Count)" }
$taskOwned = @($taskRoots)
for ($index = 0; $index -lt $taskOwned.Count; $index++) {
    $taskParent = $taskOwned[$index].ProcessId
    $taskOwned += @($taskProcesses | Where-Object ParentProcessId -eq $taskParent)
}
$taskOwned | Select-Object ProcessId, ParentProcessId, Name
if ($Stop) {
    # Only the captured investigation tree, never the game or unrelated Java/Node.
    foreach ($taskProcess in @($taskOwned | Sort-Object ProcessId -Descending)) {
        $taskCurrent = Get-CimInstance Win32_Process -Filter "ProcessId=$($taskProcess.ProcessId)" -ErrorAction SilentlyContinue
        if ($taskCurrent -and $taskCurrent.CreationDate -eq $taskProcess.CreationDate) {
            Stop-Process -Id $taskCurrent.ProcessId -Force -ErrorAction SilentlyContinue
        }
    }
}
