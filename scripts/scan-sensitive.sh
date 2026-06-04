#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

patterns=(
  'sk-[A-Za-z0-9_-]{8,}'
  'Bearer [A-Za-z0-9._:-]{16,}'
  '10\.[0-9]+\.[0-9]+\.[0-9]+'
  '172\.(1[6-9]|2[0-9]|3[0-1])\.[0-9]+\.[0-9]+'
  '192\.168\.[0-9]+\.[0-9]+'
  '/Users/[^ ]+'
  '/home/[^ ]+'
  '/vol[0-9]+/'
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

if [[ -n "${ANIMEM_PUBLIC_DENYLIST:-}" ]]; then
  if [[ ! -f "$ANIMEM_PUBLIC_DENYLIST" ]]; then
    echo "ANIMEM_PUBLIC_DENYLIST is not a file" >&2
    exit 64
  fi
  while IFS= read -r term; do
    [[ -z "$term" || "$term" == \#* ]] && continue
    if rg -n --fixed-strings "${scan_args[@]}" -- "$term" .; then
      status=1
    fi
  done < "$ANIMEM_PUBLIC_DENYLIST"
fi

package_list="$(mktemp "${TMPDIR:-/tmp}/animem-package-list.XXXXXX")"
trap 'rm -f "$package_list"' EXIT
cargo package --allow-dirty --no-verify --list >"$package_list"
bash scripts/check-source-bundle-allowlist "$package_list" || status=1

if [[ "$status" -ne 0 ]]; then
  echo "sensitive scan failed" >&2
  exit "$status"
fi

echo "sensitive scan passed"
