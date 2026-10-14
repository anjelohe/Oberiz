$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $env:SystemRoot 'System32/WindowsPowerShell/v1.0/Modules/Microsoft.PowerShell.Security/Microsoft.PowerShell.Security.psd1') -ErrorAction Stop
$taskTarget = Get-Item -LiteralPath $env:OBERIZ_ACL_TARGET -Force
if (-not $taskTarget.PSIsContainer -or ($taskTarget.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Not an ordinary private directory' }
$taskCurrent = [Security.Principal.WindowsIdentity]::GetCurrent().User
$taskOwner = (Get-Acl -LiteralPath $taskTarget.FullName).Owner
$taskSids = @('S-1-5-18', 'S-1-5-32-544', 'S-1-5-19', $taskCurrent.Value)
$taskSids += ([Security.Principal.NTAccount]::new($taskOwner)).Translate([Security.Principal.SecurityIdentifier]).Value
$taskServiceAccount = (Get-ItemProperty -LiteralPath 'HKLM:\SYSTEM\CurrentControlSet\Services\Oberiz' -Name ObjectName -ErrorAction SilentlyContinue).ObjectName
if ($taskServiceAccount -and $taskServiceAccount -ne 'LocalSystem') {
    $taskSids += ([Security.Principal.NTAccount]::new($taskServiceAccount)).Translate([Security.Principal.SecurityIdentifier]).Value
}
$taskAclExe = Join-Path $env:SystemRoot 'System32/icacls.exe'
$taskAllowed = @($taskSids | Select-Object -Unique)
foreach ($taskSid in ($taskSids | Select-Object -Unique)) {
    & $taskAclExe $taskTarget.FullName /grant:r "*${taskSid}:(OI)(CI)F" /Q | Out-Null
    if ($LASTEXITCODE) { throw 'Cannot grant private storage access' }
}
& $taskAclExe $taskTarget.FullName /inheritance:r /Q | Out-Null
if ($LASTEXITCODE) { throw 'Cannot disable public inheritance' }
foreach ($taskOldRule in (Get-Acl -LiteralPath $taskTarget.FullName).Access) {
    $taskOldSid = $taskOldRule.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value
    if ($taskOldSid -notin $taskAllowed) {
        & $taskAclExe $taskTarget.FullName /remove "*$taskOldSid" /Q | Out-Null
        if ($LASTEXITCODE) { throw 'Cannot remove public storage grant' }
    }
}
# Existing files must lose their old explicit/inherited public grants too.
$taskPending = [Collections.Generic.Queue[IO.DirectoryInfo]]::new()
$taskPending.Enqueue($taskTarget)
while ($taskPending.Count) {
    $taskDirectory = $taskPending.Dequeue()
    foreach ($taskEntry in (Get-ChildItem -LiteralPath $taskDirectory.FullName -Force)) {
        if ($taskEntry.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Private storage contains a reparse point; refusing to follow it' }
        if ($taskEntry.PSIsContainer) { $taskPending.Enqueue($taskEntry) }
        $taskExistingAcl = Get-Acl -LiteralPath $taskEntry.FullName
        $taskUnexpected = @($taskExistingAcl.Access | Where-Object { $_.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value -notin $taskAllowed })
        if (-not $taskExistingAcl.AreAccessRulesProtected -and -not $taskUnexpected.Count) { continue }
        & $taskAclExe $taskEntry.FullName /reset /Q | Out-Null
        if ($LASTEXITCODE) { throw 'Cannot reset private child ACL' }
    }
}
