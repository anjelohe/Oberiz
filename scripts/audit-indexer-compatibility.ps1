param(
    [string]$DefinitionsPath = (Join-Path $PSScriptRoot '..\config\indexers')
)

# This is intentionally a conservative static audit. It does not claim that a
# tracker is usable without credentials; it finds Cardigann features that the
# current Oberiz runtime cannot fully emulate and therefore need either parser
# support or a real-account test before being advertised as compatible.
$rules = @(
    @{ Name = 'CSRF selector inputs'; Pattern = '(?m)^\s*selectorinputs\s*:'; Severity = 'needs parser support' },
    @{ Name = 'CAPTCHA'; Pattern = '(?i)captcha|recaptcha|hcaptcha'; Severity = 'manual / unsupported' },
    @{ Name = 'Cloudflare'; Pattern = '(?i)cloudflare|cf_clearance'; Severity = 'manual / unsupported' },
    @{ Name = 'dateparse filter'; Pattern = '(?m)name:\s*dateparse\b'; Severity = 'needs parser support' },
    @{ Name = 'JavaScript requirement'; Pattern = '(?i)javascript|jschallenge'; Severity = 'manual / unsupported' }
)

$files = Get-ChildItem -Path $DefinitionsPath -Recurse -File -Include *.yml,*.yaml |
    Sort-Object FullName
$report = foreach ($file in $files) {
    $content = Get-Content -LiteralPath $file.FullName -Raw
    $id = if ($content -match '(?m)^id:\s*(.+)$') { $Matches[1].Trim() } else { $file.BaseName }
    $findings = foreach ($rule in $rules) {
        if ($content -match $rule.Pattern) { "$($rule.Name) [$($rule.Severity)]" }
    }
    [pscustomobject]@{
        Id       = $id
        File     = $file.FullName.Substring((Resolve-Path $DefinitionsPath).Path.Length).TrimStart('\','/')
        Status   = if ($findings) { 'review required' } else { 'no known static blocker' }
        Findings = $findings -join '; '
    }
}

$report | Format-Table -AutoSize
"`nDefinitions: $($report.Count)"
"Review required: $(@($report | Where-Object Status -eq 'review required').Count)"
"No known static blocker: $(@($report | Where-Object Status -eq 'no known static blocker').Count)"
