#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

patterns=(
  'sk-[A-Za-z0-9_-]{8,}'
  'Bearer [A-Za-z0-9._:-]{16,}'
  '10\.[0-9]+\.[0-9]+\.[0-9]+'
  '192\.168\.[0-9]+\.[0-9]+'
  '/Users/[^ ]+'
  '/home/[^ ]+'
  '/vol[0-9]+/'
  'axis'
  'memory-flow'
  'yihub'
  'oMLX'
  'Qwen'
  'mf-(rust|mcp|server|worker|core|cli)'
  '综合二科'
  '云浮'
  '市委'
)

scan_args=(
  --hidden
  --glob '!target/**'
  --glob '!.git/**'
  --glob '!Cargo.lock'
  --glob '!scripts/scan-sensitive.sh'
)

status=0
for pattern in "${patterns[@]}"; do
  if rg -n "${scan_args[@]}" "$pattern" .; then
    status=1
  fi
done

if [[ "$status" -ne 0 ]]; then
  echo "sensitive scan failed" >&2
  exit "$status"
fi

echo "sensitive scan passed"
