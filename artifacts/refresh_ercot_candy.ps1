param(
    [string]$InputPath = "data\This section contains important mes.txt",
    [string]$OutputDir = "data"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Normalize-Notice {
    param([string]$Text)
    if ([string]::IsNullOrWhiteSpace($Text)) { return "" }
    $normalized = $Text.Trim().ToLowerInvariant()
    $normalized = $normalized -replace "\s+", " "
    $normalized = $normalized -replace "[\.\s]+$", ""
    return $normalized
}

if (-not (Test-Path -LiteralPath $InputPath)) {
    throw "Input file not found: $InputPath"
}

if (-not (Test-Path -LiteralPath $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir | Out-Null
}

$lines = Get-Content -LiteralPath $InputPath
$rows = $lines | Where-Object { $_ -match '^(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)\s' }

$records = foreach ($r in $rows) {
    $parts = $r -split "`t"
    if ($parts.Count -lt 2) { continue }

    $dt = $null
    try {
        $dt = [datetime]::ParseExact($parts[0].Trim(), "MMM d, yyyy h:mm:ss tt", [System.Globalization.CultureInfo]::InvariantCulture)
    } catch {
        continue
    }

    [PSCustomObject]@{
        DateTime   = $parts[0].Trim()
        ParsedDate = $dt
        Notice     = $parts[1].Trim()
        Type       = if ($parts.Count -gt 2) { $parts[2].Trim() } else { "" }
        Status     = if ($parts.Count -gt 3) { $parts[3].Trim() } else { "" }
    }
}

if (-not $records -or $records.Count -eq 0) {
    throw "No message rows were parsed from $InputPath"
}

$operationsPath = Join-Path $OutputDir "operations_messages.csv"
$records |
    Select-Object DateTime, Notice, Type, Status |
    Export-Csv -LiteralPath $operationsPath -NoTypeInformation -Encoding UTF8

$maxDt = ($records | Measure-Object -Property ParsedDate -Maximum).Maximum
$minDt = ($records | Measure-Object -Property ParsedDate -Minimum).Minimum
$cutoff = $maxDt.AddHours(-72)

$activeRecent = $records |
    Where-Object { $_.Status -eq "Active" -and $_.ParsedDate -ge $cutoff } |
    Sort-Object ParsedDate -Descending |
    Select-Object DateTime, Notice, Type, Status
$activeRecentPath = Join-Path $OutputDir "active_recent.csv"
$activeRecent | Export-Csv -LiteralPath $activeRecentPath -NoTypeInformation -Encoding UTF8

$highPriority = $records |
    Where-Object { $_.Type -in @("Advisory", "Watch", "OCN", "Alert") } |
    Select-Object DateTime, Notice, Type, Status
$highPriorityPath = Join-Path $OutputDir "high_priority_alerts.csv"
$highPriority | Export-Csv -LiteralPath $highPriorityPath -NoTypeInformation -Encoding UTF8

$manualActions = $records |
    Where-Object { $_.Notice -match "(?i)manual action" } |
    Select-Object DateTime, Notice, Type, Status
$manualActionsPath = Join-Path $OutputDir "manual_actions.csv"
$manualActions | Export-Csv -LiteralPath $manualActionsPath -NoTypeInformation -Encoding UTF8

$suddenLossEvents = $records |
    Where-Object { $_.Notice -match "(?i)sudden loss of generation" } |
    Select-Object DateTime, Notice, Type, Status
$suddenLossPath = Join-Path $OutputDir "sudden_loss_events.csv"
$suddenLossEvents | Export-Csv -LiteralPath $suddenLossPath -NoTypeInformation -Encoding UTF8

$cancelPrefix = "ERCOT has cancelled the following notice:"
$cancelledTargets = New-Object System.Collections.Generic.HashSet[string]
foreach ($rec in $records) {
    if ($rec.Notice.StartsWith($cancelPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        $target = $rec.Notice.Substring($cancelPrefix.Length).Trim()
        $normalizedTarget = Normalize-Notice -Text $target
        if ($normalizedTarget) {
            [void]$cancelledTargets.Add($normalizedTarget)
        }
    }
}

$currentOpenIssues = $records |
    Where-Object { $_.Status -eq "Active" } |
    Where-Object {
        $normalizedNotice = Normalize-Notice -Text $_.Notice
        -not $cancelledTargets.Contains($normalizedNotice)
    } |
    Sort-Object ParsedDate -Descending |
    Select-Object DateTime, Notice, Type, Status
$openIssuesPath = Join-Path $OutputDir "current_open_issues.csv"
$currentOpenIssues | Export-Csv -LiteralPath $openIssuesPath -NoTypeInformation -Encoding UTF8

$byType = $records |
    Group-Object -Property Type |
    Sort-Object Count -Descending

$reportPath = Join-Path $OutputDir "ercot_candy_report.md"
$topOpen = $currentOpenIssues | Select-Object -First 12
$reportLines = New-Object System.Collections.Generic.List[string]
$reportLines.Add("# ERCOT Candy Report")
$reportLines.Add("")
$reportLines.Add("Generated: $([datetime]::Now.ToString('yyyy-MM-dd HH:mm:ss zzz'))")
$reportLines.Add("Source: $InputPath")
$reportLines.Add("Data window: $($minDt.ToString('yyyy-MM-dd HH:mm:ss')) to $($maxDt.ToString('yyyy-MM-dd HH:mm:ss'))")
$reportLines.Add("")
$reportLines.Add("## Snapshot")
$reportLines.Add("")
$reportLines.Add("- Total messages: $($records.Count)")
$reportLines.Add("- Active messages: $(($records | Where-Object { $_.Status -eq 'Active' }).Count)")
$reportLines.Add("- Cancelled messages: $(($records | Where-Object { $_.Status -eq 'Cancelled' }).Count)")
$reportLines.Add("- Current open issues (active without matched cancellation): $($currentOpenIssues.Count)")
$reportLines.Add("- High-priority alerts (`Advisory/Watch/OCN/Alert`): $($highPriority.Count)")
$reportLines.Add("- Manual action notices: $($manualActions.Count)")
$reportLines.Add("- Sudden-loss notices: $($suddenLossEvents.Count)")
$reportLines.Add("")
$reportLines.Add("## Messages By Type")
$reportLines.Add("")
foreach ($t in $byType) {
    $typeName = if ([string]::IsNullOrWhiteSpace($t.Name)) { "(blank)" } else { $t.Name }
    $reportLines.Add("- ${typeName}: $($t.Count)")
}
$reportLines.Add("")
$reportLines.Add("## Top Current Open Issues")
$reportLines.Add("")
if ($topOpen.Count -eq 0) {
    $reportLines.Add("- No open issues identified.")
} else {
    foreach ($item in $topOpen) {
        $reportLines.Add("- $($item.DateTime) | $($item.Type) | $($item.Notice)")
    }
}
$reportLines.Add("")
$reportLines.Add("## Output Files")
$reportLines.Add("")
$reportLines.Add("- operations_messages.csv")
$reportLines.Add("- active_recent.csv")
$reportLines.Add("- high_priority_alerts.csv")
$reportLines.Add("- manual_actions.csv")
$reportLines.Add("- sudden_loss_events.csv")
$reportLines.Add("- current_open_issues.csv")
$reportLines.Add("- ercot_candy_report.md")

$reportLines | Set-Content -LiteralPath $reportPath -Encoding UTF8

Write-Output "Wrote: $operationsPath"
Write-Output "Wrote: $activeRecentPath"
Write-Output "Wrote: $highPriorityPath"
Write-Output "Wrote: $manualActionsPath"
Write-Output "Wrote: $suddenLossPath"
Write-Output "Wrote: $openIssuesPath"
Write-Output "Wrote: $reportPath"
