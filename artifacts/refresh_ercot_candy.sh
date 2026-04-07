#!/usr/bin/env bash
set -euo pipefail

INPUT_PATH="${1:-data/This section contains important mes.txt}"
OUTPUT_DIR="${2:-data}"

python3 - "$INPUT_PATH" "$OUTPUT_DIR" <<'PY'
import csv
import datetime as dt
import os
import re
import sys
from collections import Counter

input_path = sys.argv[1]
output_dir = sys.argv[2]

if not os.path.exists(input_path):
    raise SystemExit(f"Input file not found: {input_path}")
os.makedirs(output_dir, exist_ok=True)

month_prefix = re.compile(r"^(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)\s")

records = []
with open(input_path, "r", encoding="utf-8", errors="replace") as f:
    for raw in f:
        line = raw.rstrip("\n")
        if not month_prefix.match(line):
            continue
        parts = line.split("\t")
        if len(parts) < 2:
            continue
        date_text = parts[0].strip()
        try:
            parsed = dt.datetime.strptime(date_text, "%b %d, %Y %I:%M:%S %p")
        except ValueError:
            try:
                parsed = dt.datetime.strptime(date_text, "%b %e, %Y %I:%M:%S %p")
            except ValueError:
                continue
        records.append(
            {
                "DateTime": date_text,
                "ParsedDate": parsed,
                "Notice": parts[1].strip() if len(parts) > 1 else "",
                "Type": parts[2].strip() if len(parts) > 2 else "",
                "Status": parts[3].strip() if len(parts) > 3 else "",
            }
        )

if not records:
    raise SystemExit("No rows parsed from input.")

def write_csv(path, rows):
    with open(path, "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=["DateTime", "Notice", "Type", "Status"])
        w.writeheader()
        for r in rows:
            w.writerow(
                {
                    "DateTime": r["DateTime"],
                    "Notice": r["Notice"],
                    "Type": r["Type"],
                    "Status": r["Status"],
                }
            )

def norm_notice(s: str) -> str:
    s = (s or "").strip().lower()
    s = re.sub(r"\s+", " ", s)
    s = re.sub(r"[\.\s]+$", "", s)
    return s

records_sorted = sorted(records, key=lambda x: x["ParsedDate"], reverse=True)
max_dt = max(r["ParsedDate"] for r in records)
min_dt = min(r["ParsedDate"] for r in records)
cutoff = max_dt - dt.timedelta(hours=72)

operations_path = os.path.join(output_dir, "operations_messages.csv")
write_csv(operations_path, records_sorted)

active_recent = [r for r in records_sorted if r["Status"] == "Active" and r["ParsedDate"] >= cutoff]
high_priority = [r for r in records_sorted if r["Type"] in {"Advisory", "Watch", "OCN", "Alert"}]
manual_actions = [r for r in records_sorted if re.search(r"manual action", r["Notice"], re.IGNORECASE)]
sudden_loss = [r for r in records_sorted if re.search(r"sudden loss of generation", r["Notice"], re.IGNORECASE)]

cancel_prefix = "ercot has cancelled the following notice:"
cancelled_targets = set()
for r in records_sorted:
    n = r["Notice"].strip()
    if n.lower().startswith(cancel_prefix):
        cancelled_targets.add(norm_notice(n[len(cancel_prefix):].strip()))

current_open = [
    r for r in records_sorted
    if r["Status"] == "Active" and norm_notice(r["Notice"]) not in cancelled_targets
]

write_csv(os.path.join(output_dir, "active_recent.csv"), active_recent)
write_csv(os.path.join(output_dir, "high_priority_alerts.csv"), high_priority)
write_csv(os.path.join(output_dir, "manual_actions.csv"), manual_actions)
write_csv(os.path.join(output_dir, "sudden_loss_events.csv"), sudden_loss)
write_csv(os.path.join(output_dir, "current_open_issues.csv"), current_open)

type_counts = Counter(r["Type"] or "(blank)" for r in records_sorted)

report_path = os.path.join(output_dir, "ercot_candy_report.md")
with open(report_path, "w", encoding="utf-8") as f:
    f.write("# ERCOT Candy Report\n\n")
    f.write(f"Generated: {dt.datetime.now().astimezone().strftime('%Y-%m-%d %H:%M:%S %z')}\n")
    f.write(f"Source: {input_path}\n")
    f.write(f"Data window: {min_dt.strftime('%Y-%m-%d %H:%M:%S')} to {max_dt.strftime('%Y-%m-%d %H:%M:%S')}\n\n")
    f.write("## Snapshot\n\n")
    f.write(f"- Total messages: {len(records_sorted)}\n")
    f.write(f"- Active messages: {sum(1 for r in records_sorted if r['Status'] == 'Active')}\n")
    f.write(f"- Cancelled messages: {sum(1 for r in records_sorted if r['Status'] == 'Cancelled')}\n")
    f.write(f"- Current open issues (active without matched cancellation): {len(current_open)}\n")
    f.write(f"- High-priority alerts (`Advisory/Watch/OCN/Alert`): {len(high_priority)}\n")
    f.write(f"- Manual action notices: {len(manual_actions)}\n")
    f.write(f"- Sudden-loss notices: {len(sudden_loss)}\n\n")
    f.write("## Messages By Type\n\n")
    for t, c in type_counts.most_common():
        f.write(f"- {t}: {c}\n")
    f.write("\n## Top Current Open Issues\n\n")
    for row in current_open[:12]:
        f.write(f"- {row['DateTime']} | {row['Type']} | {row['Notice']}\n")
    f.write("\n## Output Files\n\n")
    for name in [
        "operations_messages.csv",
        "active_recent.csv",
        "high_priority_alerts.csv",
        "manual_actions.csv",
        "sudden_loss_events.csv",
        "current_open_issues.csv",
        "ercot_candy_report.md",
    ]:
        f.write(f"- {name}\n")

print(f"Wrote {len(records_sorted)} messages to {output_dir}")
PY
