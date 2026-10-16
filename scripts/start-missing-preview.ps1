param([ValidateRange(1024,65535)][int]$Port=2033,[switch]$Build)
$ErrorActionPreference='Stop'
$taskRoot=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$taskBackend=Join-Path $taskRoot 'backend'
$taskExe=Join-Path $taskBackend 'target/debug/oberiz.exe'
if($Build){
    & cargo build --locked --manifest-path (Join-Path $taskBackend 'Cargo.toml')
    if($LASTEXITCODE -ne 0){throw 'Backend build failed'}
    & npm run build --prefix (Join-Path $taskRoot 'frontend')
    if($LASTEXITCODE -ne 0){throw 'Frontend build failed'}
}
if(!(Test-Path -LiteralPath $taskExe)){throw 'Build first: scripts/start-missing-preview.ps1 -Build'}
$taskProbe=[System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback,$Port)
try{$taskProbe.Start()}finally{$taskProbe.Stop()}
$taskPreview=Join-Path $taskBackend 'backups/missing-preview'
New-Item -ItemType Directory -Force -Path $taskPreview | Out-Null
$taskInfo=[System.Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName=$taskExe
$taskInfo.WorkingDirectory=$taskBackend
$taskInfo.UseShellExecute=$false
$taskInfo.CreateNoWindow=$true
$taskInfo.WindowStyle=[System.Diagnostics.ProcessWindowStyle]::Hidden
$taskInfo.Environment['OBERIZ_DATA_DIR']=Join-Path $taskPreview 'data'
$taskInfo.Environment['OBERIZ_CONFIG_DIR']=Join-Path $taskPreview 'config'
$taskInfo.Environment['OBERIZ_STATIC_DIR']=Join-Path $taskRoot 'frontend/dist'
$taskInfo.Environment['OBERIZ_BIND_ADDRESS']="127.0.0.1:$Port"
# Never reuse production data/config, install a service or expose the preview to LAN.
$taskProcess=[System.Diagnostics.Process]::Start($taskInfo)
[pscustomobject]@{Url="http://127.0.0.1:$Port";ProcessId=$taskProcess.Id;Data=$taskPreview;StopCommand="Stop-Process -Id $($taskProcess.Id)"}
