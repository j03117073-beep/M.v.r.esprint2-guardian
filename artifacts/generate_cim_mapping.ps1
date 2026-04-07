Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$root = (Resolve-Path '.').Path
$unpack = Join-Path $root 'artifacts/xlsx_unpacked'
$sharedPath = Join-Path $unpack 'xl/sharedStrings.xml'

function Escape-RustString([string]$s) {
    if ($null -eq $s) { return '' }
    $s = $s -replace '\\', '\\\\'
    $s = $s -replace '"', '\\"'
    $s = $s -replace "`r", ''
    $s = $s -replace "`n", '\\n'
    return $s
}

function To-Opt([string]$s) {
    if ([string]::IsNullOrWhiteSpace($s)) { return 'None' }
    return ('Some("{0}")' -f (Escape-RustString $s.Trim()))
}

$sharedXml = [xml](Get-Content -Raw $sharedPath)
$sns = New-Object System.Xml.XmlNamespaceManager($sharedXml.NameTable)
$sns.AddNamespace('d', 'http://schemas.openxmlformats.org/spreadsheetml/2006/main')
$shared = New-Object System.Collections.Generic.List[string]
foreach ($si in $sharedXml.SelectNodes('//d:si', $sns)) {
    $tnodes = $si.SelectNodes('.//d:t', $sns)
    $txt = (($tnodes | ForEach-Object { $_.'#text' }) -join '')
    $shared.Add($txt)
}

function Get-CellText($c, $ns, $shared) {
    $t = $c.t
    $v = $c.SelectSingleNode('d:v', $ns)
    $is = $c.SelectSingleNode('d:is/d:t', $ns)
    if ($t -eq 's') {
        if ($null -eq $v) { return '' }
        return $shared[[int]$v.InnerText]
    }
    if ($t -eq 'inlineStr') {
        if ($null -ne $is) { return $is.InnerText }
        return ''
    }
    if ($null -ne $v) { return $v.InnerText }
    return ''
}

function Read-SheetRows([string]$sheetPath) {
    $xml = [xml](Get-Content -Raw $sheetPath)
    $ns = New-Object System.Xml.XmlNamespaceManager($xml.NameTable)
    $ns.AddNamespace('d', 'http://schemas.openxmlformats.org/spreadsheetml/2006/main')

    $rows = @()
    foreach ($row in $xml.SelectNodes('//d:sheetData/d:row', $ns)) {
        $cells = @{}
        foreach ($c in $row.SelectNodes('d:c', $ns)) {
            $r = [string]$c.r
            $col = ([regex]::Match($r, '^[A-Z]+')).Value
            $cells[$col] = Get-CellText $c $ns $shared
        }
        $rows += ,@{ RowNum = [int]$row.r; Cells = $cells }
    }
    return $rows
}

$mappingRows = Read-SheetRows (Join-Path $unpack 'xl/worksheets/sheet1.xml')
$cim10Rows = Read-SheetRows (Join-Path $unpack 'xl/worksheets/sheet2.xml')
$cim16Rows = Read-SheetRows (Join-Path $unpack 'xl/worksheets/sheet3.xml')
$cim16AttrRows = Read-SheetRows (Join-Path $unpack 'xl/worksheets/sheet4.xml')
$cim16TypeRows = Read-SheetRows (Join-Path $unpack 'xl/worksheets/sheet5.xml')

$sb = New-Object System.Text.StringBuilder
[void]$sb.AppendLine('// Auto-generated from CIM10_to_CIM16_Attribute_Mapping_02132026.xlsx')
[void]$sb.AppendLine('// Do not edit manually.')
[void]$sb.AppendLine()
[void]$sb.AppendLine('pub struct MappingEntry {')
[void]$sb.AppendLine('    pub type_ns_cim10: Option<&''static str>,')
[void]$sb.AppendLine('    pub instance_type_cim10: Option<&''static str>,')
[void]$sb.AppendLine('    pub attr_ns_cim10: Option<&''static str>,')
[void]$sb.AppendLine('    pub attr_source_cim10: Option<&''static str>,')
[void]$sb.AppendLine('    pub attr_name_cim10: Option<&''static str>,')
[void]$sb.AppendLine('    pub data_type_cim10: Option<&''static str>,')
[void]$sb.AppendLine('    pub type_ns_cim16: Option<&''static str>,')
[void]$sb.AppendLine('    pub instance_type_cim16: Option<&''static str>,')
[void]$sb.AppendLine('    pub attr_ns_cim16: Option<&''static str>,')
[void]$sb.AppendLine('    pub attr_source_cim16: Option<&''static str>,')
[void]$sb.AppendLine('    pub attr_name_cim16: Option<&''static str>,')
[void]$sb.AppendLine('    pub data_type_cim16: Option<&''static str>,')
[void]$sb.AppendLine('    pub mapping_notes: Option<&''static str>,')
[void]$sb.AppendLine('}')
[void]$sb.AppendLine()
[void]$sb.AppendLine('pub struct Cim10Entry {')
[void]$sb.AppendLine('    pub type_ns_cim10: Option<&''static str>,')
[void]$sb.AppendLine('    pub instance_type_cim10: Option<&''static str>,')
[void]$sb.AppendLine('    pub attr_ns_cim10: Option<&''static str>,')
[void]$sb.AppendLine('    pub attr_source_cim10: Option<&''static str>,')
[void]$sb.AppendLine('    pub attr_name_cim10: Option<&''static str>,')
[void]$sb.AppendLine('    pub data_type_cim10: Option<&''static str>,')
[void]$sb.AppendLine('}')
[void]$sb.AppendLine()
[void]$sb.AppendLine('pub struct Cim16Entry {')
[void]$sb.AppendLine('    pub type_ns_cim16: Option<&''static str>,')
[void]$sb.AppendLine('    pub instance_type_cim16: Option<&''static str>,')
[void]$sb.AppendLine('    pub attr_ns_cim16: Option<&''static str>,')
[void]$sb.AppendLine('    pub attr_source_cim16: Option<&''static str>,')
[void]$sb.AppendLine('    pub attr_name_cim16: Option<&''static str>,')
[void]$sb.AppendLine('    pub data_type_cim16: Option<&''static str>,')
[void]$sb.AppendLine('}')
[void]$sb.AppendLine()
[void]$sb.AppendLine('pub struct Cim16AttrDesc {')
[void]$sb.AppendLine('    pub type_name: Option<&''static str>,')
[void]$sb.AppendLine('    pub attr_name: Option<&''static str>,')
[void]$sb.AppendLine('    pub help_message: Option<&''static str>,')
[void]$sb.AppendLine('}')
[void]$sb.AppendLine()
[void]$sb.AppendLine('pub struct Cim16TypeDesc {')
[void]$sb.AppendLine('    pub type_name: Option<&''static str>,')
[void]$sb.AppendLine('    pub help_message: Option<&''static str>,')
[void]$sb.AppendLine('}')
[void]$sb.AppendLine()

[void]$sb.AppendLine('pub static MAPPING_DATA: &[MappingEntry] = &[')
$mappingDataRows = $mappingRows | Where-Object { $_.RowNum -gt 1 }
foreach ($r in $mappingDataRows) {
    $c = $r.Cells
    [void]$sb.AppendLine('    MappingEntry {')
    [void]$sb.AppendLine(('        type_ns_cim10: {0},' -f (To-Opt $c['A'])))
    [void]$sb.AppendLine(('        instance_type_cim10: {0},' -f (To-Opt $c['B'])))
    [void]$sb.AppendLine(('        attr_ns_cim10: {0},' -f (To-Opt $c['C'])))
    [void]$sb.AppendLine(('        attr_source_cim10: {0},' -f (To-Opt $c['D'])))
    [void]$sb.AppendLine(('        attr_name_cim10: {0},' -f (To-Opt $c['E'])))
    [void]$sb.AppendLine(('        data_type_cim10: {0},' -f (To-Opt $c['F'])))
    [void]$sb.AppendLine(('        type_ns_cim16: {0},' -f (To-Opt $c['G'])))
    [void]$sb.AppendLine(('        instance_type_cim16: {0},' -f (To-Opt $c['H'])))
    [void]$sb.AppendLine(('        attr_ns_cim16: {0},' -f (To-Opt $c['I'])))
    [void]$sb.AppendLine(('        attr_source_cim16: {0},' -f (To-Opt $c['J'])))
    [void]$sb.AppendLine(('        attr_name_cim16: {0},' -f (To-Opt $c['K'])))
    [void]$sb.AppendLine(('        data_type_cim16: {0},' -f (To-Opt $c['L'])))
    [void]$sb.AppendLine(('        mapping_notes: {0},' -f (To-Opt $c['M'])))
    [void]$sb.AppendLine('    },')
}
[void]$sb.AppendLine('];')
[void]$sb.AppendLine()

[void]$sb.AppendLine('pub static MAPPING_DATA_WITH_NOTES: &[MappingEntry] = &[')
foreach ($r in $mappingDataRows) {
    $c = $r.Cells
    if (-not [string]::IsNullOrWhiteSpace($c['M'])) {
        [void]$sb.AppendLine('    MappingEntry {')
        [void]$sb.AppendLine(('        type_ns_cim10: {0},' -f (To-Opt $c['A'])))
        [void]$sb.AppendLine(('        instance_type_cim10: {0},' -f (To-Opt $c['B'])))
        [void]$sb.AppendLine(('        attr_ns_cim10: {0},' -f (To-Opt $c['C'])))
        [void]$sb.AppendLine(('        attr_source_cim10: {0},' -f (To-Opt $c['D'])))
        [void]$sb.AppendLine(('        attr_name_cim10: {0},' -f (To-Opt $c['E'])))
        [void]$sb.AppendLine(('        data_type_cim10: {0},' -f (To-Opt $c['F'])))
        [void]$sb.AppendLine(('        type_ns_cim16: {0},' -f (To-Opt $c['G'])))
        [void]$sb.AppendLine(('        instance_type_cim16: {0},' -f (To-Opt $c['H'])))
        [void]$sb.AppendLine(('        attr_ns_cim16: {0},' -f (To-Opt $c['I'])))
        [void]$sb.AppendLine(('        attr_source_cim16: {0},' -f (To-Opt $c['J'])))
        [void]$sb.AppendLine(('        attr_name_cim16: {0},' -f (To-Opt $c['K'])))
        [void]$sb.AppendLine(('        data_type_cim16: {0},' -f (To-Opt $c['L'])))
        [void]$sb.AppendLine(('        mapping_notes: {0},' -f (To-Opt $c['M'])))
        [void]$sb.AppendLine('    },')
    }
}
[void]$sb.AppendLine('];')
[void]$sb.AppendLine()

[void]$sb.AppendLine('pub static CIM10_DATA: &[Cim10Entry] = &[')
foreach ($r in ($cim10Rows | Where-Object { $_.RowNum -gt 1 })) {
    $c = $r.Cells
    [void]$sb.AppendLine('    Cim10Entry {')
    [void]$sb.AppendLine(('        type_ns_cim10: {0},' -f (To-Opt $c['A'])))
    [void]$sb.AppendLine(('        instance_type_cim10: {0},' -f (To-Opt $c['B'])))
    [void]$sb.AppendLine(('        attr_ns_cim10: {0},' -f (To-Opt $c['C'])))
    [void]$sb.AppendLine(('        attr_source_cim10: {0},' -f (To-Opt $c['D'])))
    [void]$sb.AppendLine(('        attr_name_cim10: {0},' -f (To-Opt $c['E'])))
    [void]$sb.AppendLine(('        data_type_cim10: {0},' -f (To-Opt $c['F'])))
    [void]$sb.AppendLine('    },')
}
[void]$sb.AppendLine('];')
[void]$sb.AppendLine()

[void]$sb.AppendLine('pub static CIM16_DATA: &[Cim16Entry] = &[')
foreach ($r in ($cim16Rows | Where-Object { $_.RowNum -gt 1 })) {
    $c = $r.Cells
    [void]$sb.AppendLine('    Cim16Entry {')
    [void]$sb.AppendLine(('        type_ns_cim16: {0},' -f (To-Opt $c['A'])))
    [void]$sb.AppendLine(('        instance_type_cim16: {0},' -f (To-Opt $c['B'])))
    [void]$sb.AppendLine(('        attr_ns_cim16: {0},' -f (To-Opt $c['C'])))
    [void]$sb.AppendLine(('        attr_source_cim16: {0},' -f (To-Opt $c['D'])))
    [void]$sb.AppendLine(('        attr_name_cim16: {0},' -f (To-Opt $c['E'])))
    [void]$sb.AppendLine(('        data_type_cim16: {0},' -f (To-Opt $c['F'])))
    [void]$sb.AppendLine('    },')
}
[void]$sb.AppendLine('];')
[void]$sb.AppendLine()

[void]$sb.AppendLine('pub static CIM16_ATTR_DESC_DATA: &[Cim16AttrDesc] = &[')
foreach ($r in ($cim16AttrRows | Where-Object { $_.RowNum -gt 1 })) {
    $c = $r.Cells
    [void]$sb.AppendLine('    Cim16AttrDesc {')
    [void]$sb.AppendLine(('        type_name: {0},' -f (To-Opt $c['A'])))
    [void]$sb.AppendLine(('        attr_name: {0},' -f (To-Opt $c['B'])))
    [void]$sb.AppendLine(('        help_message: {0},' -f (To-Opt $c['C'])))
    [void]$sb.AppendLine('    },')
}
[void]$sb.AppendLine('];')
[void]$sb.AppendLine()

[void]$sb.AppendLine('pub static CIM16_TYPE_DESC_DATA: &[Cim16TypeDesc] = &[')
foreach ($r in ($cim16TypeRows | Where-Object { $_.RowNum -gt 1 })) {
    $c = $r.Cells
    [void]$sb.AppendLine('    Cim16TypeDesc {')
    [void]$sb.AppendLine(('        type_name: {0},' -f (To-Opt $c['A'])))
    [void]$sb.AppendLine(('        help_message: {0},' -f (To-Opt $c['B'])))
    [void]$sb.AppendLine('    },')
}
[void]$sb.AppendLine('];')

$outPath = Join-Path $root 'src/cim_mapping_data.rs'
[System.IO.File]::WriteAllText($outPath, $sb.ToString(), [System.Text.UTF8Encoding]::new($false))

$notesCount = ($mappingDataRows | Where-Object { -not [string]::IsNullOrWhiteSpace($_.Cells['M']) }).Count
Write-Output ("Generated: {0}" -f $outPath)
Write-Output ("Mapping rows: {0}" -f $mappingDataRows.Count)
Write-Output ("Notes rows: {0}" -f $notesCount)
Write-Output ("CIM10 rows: {0}" -f (($cim10Rows | Where-Object { $_.RowNum -gt 1 }).Count))
Write-Output ("CIM16 rows: {0}" -f (($cim16Rows | Where-Object { $_.RowNum -gt 1 }).Count))
Write-Output ("CIM16 attr desc rows: {0}" -f (($cim16AttrRows | Where-Object { $_.RowNum -gt 1 }).Count))
Write-Output ("CIM16 type desc rows: {0}" -f (($cim16TypeRows | Where-Object { $_.RowNum -gt 1 }).Count))
