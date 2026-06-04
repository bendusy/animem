# Dependency Map

`animem` starts as a small, pure Rust core crate.

## Required Dependencies

| Dependency | Why It Exists | Public Risk |
|---|---|---|
| `serde` | stable structs for caller-owned persistence | low |
| `serde_json` | JSON payloads for candidates and tests | low |
| `toml` | TOML profile and extension files for local maintenance configs | low |
| `sha2` | deterministic asset and section identifiers | low |
| `regex` | conservative heading and numbering detection | low |
| `thiserror` | typed library errors | low |

## Deferred Integrations

These are intentionally not part of the base crate:

- persistence backends;
- embedding/model clients;
- service runtimes;
- private deployment scripts;
- private profile runners.

Do not treat this list as a public roadmap. Any integration must be separately
scoped and reviewed before it becomes part of this repository.
