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

## Review Checklist

Before committing, run:

```bash
cargo test
./scripts/scan-sensitive.sh
```

The scanner is intentionally conservative. Fix findings in public files rather
than adding broad allowlists.
