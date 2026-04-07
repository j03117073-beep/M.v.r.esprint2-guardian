Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$root = (Resolve-Path '.').Path
$unpack = Join-Path $root 'artifacts/mora_unpacked'
$sharedPath = Join-Path $unpack 'xl/sharedStrings.xml'
$sheetPath = Join-Path $unpack 'xl/worksheets/sheet6.xml' # Resource Details
$outCsv = Join-Path $root 'artifacts/mora_june2026_resource_details.csv'

$sharedXml = [xml](Get-Content -Raw $sharedPath)
$sns = New-Object System.Xml.XmlNamespaceManager($sharedXml.NameTable)
$sns.AddNamespace('d', 'http://schemas.openxmlformats.org/spreadsheetml/2006/main')
$S = New-Object System.Collections.Generic.List[string]
foreach ($si in $sharedXml.SelectNodes('//d:si', $sns)) {
    $S.Add((($si.SelectNodes('.//d:t', $sns) | ForEach-Object { $_.'#text' }) -join ''))
}

function Get-CellVal($c, $ns) {
    $t = $c.GetAttribute('t')
    $v = $c.SelectSingleNode('d:v', $ns)
    if ($t -eq 's' -and $v) { return $S[[int]$v.InnerText] }
    if ($v) { return $v.InnerText }
    return ''
}

$sheet = [xml](Get-Content -Raw $sheetPath)
$ns = New-Object System.Xml.XmlNamespaceManager($sheet.NameTable)
$ns.AddNamespace('d', 'http://schemas.openxmlformats.org/spreadsheetml/2006/main')
$rows = $sheet.SelectNodes('//d:sheetData/d:row', $ns)

$data = @()
$currentCategory = ''

foreach ($row in $rows) {
    $rnum = [int]$row.r
    if ($rnum -lt 3) { continue }

    $vals = @{}
    foreach ($c in $row.SelectNodes('d:c', $ns)) {
        $col = ([regex]::Match([string]$c.r, '^[A-Z]+')).Value
        $vals[$col] = Get-CellVal $c $ns
    }

    $a = ($vals['A'] | ForEach-Object { $_.ToString().Trim() })
    $b = ($vals['B'] | ForEach-Object { $_.ToString().Trim() })
    $c = ($vals['C'] | ForEach-Object { $_.ToString().Trim() })
    $d = ($vals['D'] | ForEach-Object { $_.ToString().Trim() })
    $e = ($vals['E'] | ForEach-Object { $_.ToString().Trim() })
    $f = ($vals['F'] | ForEach-Object { $_.ToString().Trim() })
    $g = ($vals['G'] | ForEach-Object { $_.ToString().Trim() })
    $h = ($vals['H'] | ForEach-Object { $_.ToString().Trim() })
    $i = ($vals['I'] | ForEach-Object { $_.ToString().Trim() })
    $j = ($vals['J'] | ForEach-Object { $_.ToString().Trim() })

    # Category row: no serial number, text in UNIT NAME
    if ([string]::IsNullOrWhiteSpace($a) -and -not [string]::IsNullOrWhiteSpace($b) -and [string]::IsNullOrWhiteSpace($d)) {
        $currentCategory = $b
        continue
    }

    # Data row: numeric serial in column A + unit code in D
    $serialParsed = 0
    $hasSerial = [int]::TryParse($a, [ref]$serialParsed)
    if ($hasSerial -and -not [string]::IsNullOrWhiteSpace($d)) {
        $data += [PSCustomObject]@{
            report_month = '2026-06'
            category = $currentCategory
            row_number = $serialParsed
            unit_name = $b
            inr = $c
            unit_code = $d
            county = $e
            fuel = $f
            zone = $g
            in_service_year = $h
            installed_capacity_mw = $i
            mora_capacity_mw = $j
        }
    }
}

$data | Export-Csv -Path $outCsv -NoTypeInformation -Encoding UTF8
Write-Output ("WROTE={0}" -f $outCsv)
Write-Output ("ROWS={0}" -f $data.Count)
