#!/usr/bin/env python3
"""
Minimal Phase III enforcement tool (dry-run by default).
Usage: python3 tools/phase3_enforce/enforce.py --dry-run

This script parses the `phase_ii/compliance/CLAIM_TRACEABILITY_MATRIX.md` and
`phase_ii/compliance/EVIDENCE_INDEX.md` files for simple table rows and produces
an `phase_iii/ENFORCEMENT_REPORT.md` summarizing link coverage.

This is a lightweight prototype for the enforcement loop.
"""

import argparse
import difflib
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PHASE3 = ROOT / 'phase_iii'

claim_file = PHASE3 / 'CLAIM_TRACEABILITY_MATRIX.md'
evidence_file = PHASE3 / 'EVIDENCE_INDEX.md'
report_file = PHASE3 / 'ENFORCEMENT_REPORT.md'
log_file = PHASE3 / 'enforcement_tx.log'
patch_file = PHASE3 / 'enforcement_patch.diff'

TABLE_ROW = re.compile(r"^\|\s*(.+?)\s*\|\s*(.+?)\s*\|\s*(.+?)\s*\|\s*(.+?)\s*\|.*$")


def parse_claims(path):
    claims = []
    if not path.exists():
        return claims
    with path.open() as f:
        in_table = False
        for line in f:
            if line.strip().startswith('|---'):
                in_table = True
                continue
            if in_table:
                if line.strip().startswith('|'):
                    m = TABLE_ROW.match(line)
                    if m:
                        # claim is first column; remove backticks and clean whitespace
                        claim = m.group(1).strip()
                        # remove backticks: `text` -> text
                        claim = re.sub(r'`([^`]*)`', r'\1', claim)
                        claim = claim.strip()
                        if claim:
                            claims.append(claim)
                else:
                    break
    return claims


def parse_evidence(path):
    evidence = []
    if not path.exists():
        return evidence
    with path.open() as f:
        in_table = False
        header_found = False
        for line in f:
            if line.strip().startswith('|---'):
                in_table = True
                header_found = True
                continue
            if in_table:
                if line.strip().startswith('|'):
                    # split by | and extract columns
                    raw_parts = line.split('|')[1:]  # skip first empty element
                    if len(raw_parts) > 5:  # need at least 6 columns
                        art = raw_parts[0].strip()
                        loc = raw_parts[1].strip()
                        desc = raw_parts[2].strip() if len(raw_parts) > 2 else ''
                        src_type = raw_parts[3].strip() if len(raw_parts) > 3 else ''
                        reproducibility = raw_parts[4].strip() if len(raw_parts) > 4 else ''
                        linked_claims_str = raw_parts[5].strip() if len(raw_parts) > 5 else ''
                        
                        # Remove backticks only from artifact/location/etc, not from linked_claims
                        art = re.sub(r'`([^`]*)`', r'\1', art)
                        loc = re.sub(r'`([^`]*)`', r'\1', loc)
                        
                        # parse linked claims from the last column (e.g., "`demo`, `pilot_demo`")
                        linked = []
                        if linked_claims_str:
                            # extract claim names from backtick-wrapped list
                            matches = re.findall(r'`([^`]+)`', linked_claims_str)
                            linked.extend(matches)
                        
                        evidence.append({
                            'artifact': art,
                            'location': loc,
                            'description': desc,
                            'source_type': src_type,
                            'reproducibility': reproducibility,
                            'linked_claims': linked
                        })
                else:
                    break
    return evidence


def write_patch(old_text, new_text, path_name):
    old_lines = old_text.splitlines(keepends=True)
    new_lines = new_text.splitlines(keepends=True)
    diff = difflib.unified_diff(
        old_lines,
        new_lines,
        fromfile=f'old/{path_name}',
        tofile=f'new/{path_name}',
        lineterm=''
    )
    return ''.join(line + '\n' for line in diff)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--dry-run', action='store_true')
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    if not args.dry_run and not args.apply:
        args.dry_run = True

    claims = parse_claims(claim_file)
    evidence = parse_evidence(evidence_file)

    linked = {c: [] for c in claims}

    # Create normalized lookup for claims (lowercase without special chars)
    def normalize_name(name):
        return name.lower().strip()
    
    claim_lookup = {normalize_name(c): c for c in claims}

    # First pass: use explicit linkage from EVIDENCE_INDEX.md "Linked Claims" column
    for art in evidence:
        for linked_claim in art.get('linked_claims', []):
            linked_claim_normalized = normalize_name(linked_claim)
            if linked_claim_normalized in claim_lookup:
                actual_claim = claim_lookup[linked_claim_normalized]
                if art not in linked[actual_claim]:  # avoid duplicates
                    linked[actual_claim].append(art)
    
    # Second pass: fuzzy matching for unlinked claims
    # Extract keywords from claim names (e.g., "demo" from "demo scenario playback")
    def extract_keywords(name):
        return set(name.lower().split())
    
    for c in claims:
        if not linked[c]:  # only link if not already linked
            keywords = extract_keywords(c)
            for art in evidence:
                art_keywords = extract_keywords(art['artifact'] + ' ' + art['location'])
                # if significant keyword overlap, link them
                if keywords & art_keywords:
                    if art not in linked[c]:  # avoid duplicates
                        linked[c].append(art)

    # Build report
    lines = []
    lines.append('# Enforcement Report (Tool)\n')
    lines.append(f'Generated: {ROOT}\n')
    lines.append(f'Claims scanned: {len(claims)}')
    lines.append(f'Evidence artifacts scanned: {len(evidence)}\n')
    
    lines.append('## Claim Linkage Summary\n')
    orphans = []
    for c in claims:
        items = linked.get(c, [])
        if items:
            lines.append(f'- {c}: linked evidence count: {len(items)}')
            for ev in items:
                lines.append(f'  - {ev["artifact"]} ({ev.get("reproducibility", "unknown")})')
        else:
            lines.append(f'- {c}: NO LINKED EVIDENCE → PENDING_EVIDENCE')
            orphans.append(c)

    lines.append(f'\n## Summary\n')
    lines.append(f'- Validated claims (linked): {len(claims) - len(orphans)}/{len(claims)}')
    lines.append(f'- Pending claims (unlinked): {len(orphans)}/{len(claims)}')
    
    lines.append(f'\n## Unlinked Claims\n')
    if orphans:
        for o in orphans:
            lines.append(f'- {o}')
    else:
        lines.append('None')
    
    lines.append(f'\n## Evidence Reproducibility Audit\n')
    reproducible = []
    non_reproducible = []
    for art in evidence:
        if art.get('reproducibility') == 'reproducible (repo-local)':
            reproducible.append(art['artifact'])
        else:
            non_reproducible.append(art['artifact'])
    
    lines.append(f'Reproducible (repo-local): {len(reproducible)}')
    for r in reproducible:
        lines.append(f'- {r}')
    
    lines.append(f'\nNon-reproducible (external/tmp): {len(non_reproducible)}')
    for n in non_reproducible:
        lines.append(f'- {n}')

    new_report = '\n'.join(lines).rstrip() + '\n'
    new_log = 'Enforcement run: {}\n'.format('apply' if args.apply else 'dry-run')

    old_report = report_file.read_text() if report_file.exists() else ''
    old_log = log_file.read_text() if log_file.exists() else ''

    patch_contents = ''
    report_diff = write_patch(old_report, new_report, 'phase_iii/ENFORCEMENT_REPORT.md')
    log_diff = write_patch(old_log, new_log, 'phase_iii/enforcement_tx.log')
    if report_diff:
        patch_contents += report_diff
    if log_diff:
        if patch_contents:
            patch_contents += '\n'
        patch_contents += log_diff

    if args.apply:
        report_file.write_text(new_report)
        log_file.write_text(new_log)
        patch_file.write_text(patch_contents or 'No changes.\n')
        print('Enforcement apply complete. Patch written to', patch_file)
    else:
        report_file.write_text(new_report)
        log_file.write_text(new_log)
        patch_file.write_text(patch_contents or 'No changes.\n')
        print('Enforcement dry-run complete. Report written to', report_file)
        print('Patch preview written to', patch_file)


if __name__ == '__main__':
    main()
