#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

scan_args=(
  --hidden
  --glob '!target/**'
  --glob '!.git/**'
  --glob '!Cargo.lock'
  --glob '!scripts/scan-sensitive.sh'
)

status=0

scan_pattern() {
  local label="$1"
  local pattern="$2"
  local matched=0
  while IFS= read -r file; do
    [[ -n "$file" ]] || continue
    printf 'sensitive pattern %s in %s\n' "$label" "$file" >&2
    matched=1
  done < <(rg -l "${scan_args[@]}" "$pattern" . || true)
  if [[ "$matched" -ne 0 ]]; then
    status=1
  fi
}

scan_pattern credential 'sk-[A-Za-z0-9_-]{8,}'
scan_pattern bearer-token 'Bearer [A-Za-z0-9._:-]{16,}'
scan_pattern private-ip-10 '10\.[0-9]+\.[0-9]+\.[0-9]+'
scan_pattern private-ip-172 '172\.(1[6-9]|2[0-9]|3[0-1])\.[0-9]+\.[0-9]+'
scan_pattern private-ip-192 '192\.168\.[0-9]+\.[0-9]+'
scan_pattern local-users-path '/Users/[^ ]+'
scan_pattern local-home-path '/home/[^ ]+'
scan_pattern local-vol-path '/vol[0-9]+'

if [[ -n "${ANIMEM_PUBLIC_DENYLIST:-}" ]]; then
  if [[ ! -f "$ANIMEM_PUBLIC_DENYLIST" ]]; then
    echo "ANIMEM_PUBLIC_DENYLIST is not a file" >&2
    exit 64
  fi
  term_index=0
  while IFS= read -r term; do
    term="${term#${term%%[![:space:]]*}}"
    term="${term%${term##*[![:space:]]}}"
    [[ -z "$term" || "$term" == \#* ]] && continue
    term_index=$((term_index + 1))
    matched=0
    while IFS= read -r file; do
      [[ -n "$file" ]] || continue
      printf 'public denylist hit term #%s in %s\n' "$term_index" "$file" >&2
      matched=1
    done < <(rg -i -l --fixed-strings "${scan_args[@]}" -- "$term" . || true)
    if [[ "$matched" -ne 0 ]]; then
      status=1
    fi
  done < "$ANIMEM_PUBLIC_DENYLIST"
fi

shopt -s nullglob
for payload in examples/provenance/*.json; do
  if ! bash scripts/check-redacted-event-payload "$payload" >/dev/null; then
    status=1
  fi
done
shopt -u nullglob

package_list="$(mktemp "${TMPDIR:-/tmp}/animem-package-list.XXXXXX")"
trap 'rm -f "$package_list"' EXIT
cargo package --allow-dirty --no-verify --list >"$package_list"
bash scripts/check-source-bundle-allowlist "$package_list" || status=1

if [[ "$status" -ne 0 ]]; then
  echo "sensitive scan failed" >&2
  exit "$status"
fi

echo "sensitive scan passed"
