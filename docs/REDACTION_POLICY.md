# Redaction Policy

Public examples must be synthetic by construction, not merely anonymized.

## Safe Fixture Names

- `Example Org`
- `Project Alpha`
- `Project Beta`
- `Policy Memo A`
- `Review Note B`

## Unsafe Fixture Content

- names of real people or organizations;
- local filesystem paths;
- LAN IP addresses and host aliases;
- document titles from real archives;
- excerpts from internal or client documents;
- hashes or IDs copied from private databases.

## Provenance Payload Gate

Public provenance JSON is checked with:

```bash
bash scripts/check-redacted-event-payload <payload.json>
```

The checker walks JSON keys and scalar values recursively, reports only leak
class and JSON pointer, and never prints raw matched values or denylist terms.
Use it for event/provenance payloads, not as a general document parser.

## Review Checklist

Before committing, run:

```bash
cargo test
bash scripts/test-redacted-event-payload
./scripts/scan-sensitive.sh
```

The scanners are intentionally conservative. Fix findings in public files rather
than adding broad allowlists.

Project-specific denylist terms must stay outside this repository. To run an
extra local denylist, set `ANIMEM_PUBLIC_DENYLIST=/path/to/denylist.txt`; do not
commit private organization, runtime, provider, or archive terms into public
scripts. Denylist hits are reported by term number and file/pointer only.
