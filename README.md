# animem

`animem` is an open-source core for document-grounded agent memory.

It provides neutral primitives for:

- registering document assets without storing private source files;
- splitting text into citeable sections;
- deriving lightweight document cards from caller-provided metadata;
- representing reviewable memory candidates before promotion;
- keeping provenance links between memory and source text.

This repository is intentionally separate from any private deployment,
database, document archive, or agent runtime. It contains no real production
cards, no internal hostnames, no private paths, and no bundled memory data.

## Current Scope

This first public slice is a pure Rust library. It does not start services,
open network connections, or require PostgreSQL.

```bash
cargo test
```

Recommended local checks before sending a change:

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
./scripts/scan-sensitive.sh
```

## Non-Goals

- No private deployment runbooks.
- No real organization dictionaries.
- No real document examples.
- No checked-in caches, embeddings, databases, or evaluation exports.
- No inherited history from private repositories.

## Dependency Policy

The core crate keeps dependencies small and auditable. Runtime integrations
such as SQL storage, HTTP APIs, embeddings, and LLM extraction should be added
behind explicit Cargo features.

See [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md).

## Data Policy

All examples and tests must use synthetic content. If a contribution needs a
fixture, use neutral names like `Example Org`, `Project Alpha`, or
`Policy Memo A`.

See [docs/REDACTION_POLICY.md](docs/REDACTION_POLICY.md).

Local data maintenance is handled through private profiles outside this
repository. See [docs/LOCAL_DATA.md](docs/LOCAL_DATA.md).

The replacement path for older `mf` maintenance code is tracked in
[docs/MIGRATION_FROM_MF.md](docs/MIGRATION_FROM_MF.md).
