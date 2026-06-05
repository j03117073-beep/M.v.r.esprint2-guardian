#!/usr/bin/env python3
"""MVRE-SPRINT1: Automated Invariant Compliance Checker

Enforces non-volatility of logs, absolute execution command mapping, and taxonomy rules.
"""

import os
import re
import sys
from pathlib import Path

FORBIDDEN_PATTERNS = [
    r"/tmp/",
    r"/var/tmp/",
    r"~\/",
]

REQUIRED_TAXONOMY_TAGS = [
    "EXECUTABLE_ARTIFACT",
    "DERIVED_REPORT",
    "STATIC_SPECIFICATION",
    "VERIFICATION_OUTPUT",
]

EVIDENCE_INDEX_PATH = Path("phase_iii/EVIDENCE_INDEX.md")
REBUILD_GRAPH_PATH = Path("phase_iii/EVIDENCE_REBUILD_GRAPH.md")


def check_volatile_paths(filepath: Path):
    violations = []
    with filepath.open("r", encoding="utf-8") as file:
        for line_num, line in enumerate(file, start=1):
            for pattern in FORBIDDEN_PATTERNS:
                if re.search(pattern, line):
                    violations.append((line_num, line.strip(), pattern))
    return violations


def parse_evidence_index(filepath: Path):
    if not filepath.exists():
        return [f"Critical Error: {filepath} is missing."]

    errors = []
    with filepath.open("r", encoding="utf-8") as file:
        lines = file.readlines()

    table_started = False
    for line in lines:
        if line.startswith("|---"):
            table_started = True
            continue
        if not table_started or not line.startswith("|"):
            continue

        columns = [entry.strip() for entry in line.strip().split("|")[1:-1]]
        if len(columns) != 7:
            continue

        artifact, location, _, taxonomy, _, command, _ = columns

        if taxonomy not in REQUIRED_TAXONOMY_TAGS:
            errors.append(f"Missing or invalid taxonomy tag for artifact '{artifact}': '{taxonomy}'")

        if not command or command.lower().startswith("manual review"):
            errors.append(f"Missing explicit reproduction command for artifact '{artifact}'")

        if "*" not in location and "," not in location and " or " not in location:
            clean_location = location.strip(' `')
            file_path = Path(clean_location)
            if not file_path.exists():
                errors.append(f"Referenced evidence path does not exist: {clean_location}")

    return errors


def main():
    print("==============================================================")
    print("      MVRE-SPRINT1 DETERMINISTIC INVARIANT COMPLIANCE GATE    ")
    print("==============================================================\n")

    has_failed = False

    for path in [EVIDENCE_INDEX_PATH, REBUILD_GRAPH_PATH]:
        if not path.exists():
            print(f"[!] Warning: {path} not found. Skipping scan.")
            has_failed = True
            continue

        print(f"[*] Scanning {path} for volatile paths...")
        violations = check_volatile_paths(path)
        if violations:
            print(f" [❌] CRITICAL INVARIANT VIOLATION in {path}:")
            for line_num, line_text, pattern in violations:
                print(f"     -> Line {line_num}: '{line_text}' | Forbidden pattern: {pattern}")
            has_failed = True
        else:
            print(f" [✔️] Path validation passed for {path}.")
        print("")

    print("[*] Validating evidence index reproducibility metadata...")
    evidence_errors = parse_evidence_index(EVIDENCE_INDEX_PATH)
    if evidence_errors:
        print(" [❌] REPRODUCIBILITY INVARIANT ERRORS DETECTED:")
        for err in evidence_errors:
            print(f"     -> {err}")
        has_failed = True
    else:
        print(" [✔️] Evidence index metadata checks passed.")

    print("\n--------------------------------------------------------------")
    if has_failed:
        print("[💥] SUMMARY: Invariant compliance failed.")
        print("    Fix the reported issues before committing.")
        print("--------------------------------------------------------------")
        sys.exit(1)
    print("[🎉] SUMMARY: Sovereign kernel invariant compliance verified.")
    print("--------------------------------------------------------------")
    sys.exit(0)


if __name__ == "__main__":
    main()
