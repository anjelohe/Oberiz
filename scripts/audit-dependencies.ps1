$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path $PSScriptRoot -Parent
$taskManifest = Join-Path $taskRoot 'backend/Cargo.toml'
# Audit packages actually used by this build, not unused optional lock entries.
$taskTree = & cargo tree --manifest-path $taskManifest --locked --all-features --prefix none --format '{p}'
if ($LASTEXITCODE) { throw 'Cannot resolve active dependencies' }
$taskPackages = @($taskTree | ForEach-Object {
    if ($_ -match '^([a-zA-Z0-9_-]+) v([0-9][^ ]*)') { "$($Matches[1]) $($Matches[2])" }
} | Sort-Object -Unique | Where-Object { $_ -notmatch '^oberiz ' })
$taskQueries = @($taskPackages | ForEach-Object {
    $taskParts = $_ -split ' ', 2
    @{ package = @{ name = $taskParts[0]; ecosystem = 'crates.io' }; version = $taskParts[1] }
})
$taskResults = Invoke-RestMethod 'https://api.osv.dev/v1/querybatch' -Method Post -ContentType 'application/json' -Body (@{queries=$taskQueries} | ConvertTo-Json -Depth 5)
$taskProblems = @()
for ($taskIndex=0; $taskIndex -lt $taskPackages.Count; $taskIndex++) {
    foreach ($taskVuln in $taskResults.results[$taskIndex].vulns) {
        $taskProblems += "$($taskPackages[$taskIndex]): $($taskVuln.id)"
    }
}
if ($taskProblems.Count) { $taskProblems | Write-Output; throw 'Dependency security advisories require review' }
Write-Output "OSV: $($taskPackages.Count) active dependency versions checked; no advisories."
& npm audit --prefix (Join-Path $taskRoot 'frontend') --audit-level=low
if ($LASTEXITCODE) { throw 'npm dependency audit failed' }
