#!/usr/bin/env bash
# Mirror runtime evidence from /tmp/phase_ii_repro/logs into repository-local evidence folder.
# Usage: ./scripts/mirror_evidence.sh --source /tmp/phase_ii_repro/logs --dest phase_ii/evidence --confirm

set -euo pipefail

SRC="/tmp/phase_ii_repro/logs"
DEST="phase_ii/evidence"
CONFIRM=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --source) SRC="$2"; shift 2;;
    --dest) DEST="$2"; shift 2;;
    --confirm) CONFIRM=1; shift;;
    --help) echo "Usage: $0 [--source <dir>] [--dest <dir>] --confirm"; exit 0;;
    *) echo "Unknown arg: $1"; exit 2;;
  esac
done

if [[ $CONFIRM -ne 1 ]]; then
  echo "Mirror is disabled by default. Provide --confirm to proceed." >&2
  echo "Example: $0 --source /tmp/phase_ii_repro/logs --dest phase_ii/evidence --confirm" >&2
  exit 3
fi

if [[ ! -d "$SRC" ]]; then
  echo "Source directory not found: $SRC" >&2
  exit 4
fi

mkdir -p "$DEST"

echo "Mirroring files from $SRC to $DEST"

shopt -s nullglob
for f in "$SRC"/*; do
  base=$(basename "$f")
  destfile="$DEST/$base"
  if [[ -e "$destfile" ]]; then
    echo "Skipping existing file: $destfile"
  else
    echo "Copying $f -> $destfile"
    cp -p "$f" "$destfile"
  fi
done

echo "Mirror complete. Review files in $DEST before committing."
