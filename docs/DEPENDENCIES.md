# Dependency Map

`animem` starts as a small, pure Rust core crate.

## Required Dependencies

| Dependency | Why It Exists | Public Risk |
|---|---|---|
| `serde` | stable structs for caller-owned persistence | low |
| `serde_json` | JSON payloads for candidates and tests | low |
| `sha2` | deterministic asset and section identifiers | low |
| `regex` | conservative heading and numbering detection | low |
| `thiserror` | typed library errors | low |

## Dev Dependencies

| Dependency | Why It Exists |
|---|---|
| `insta` | snapshot testing with synthetic fixtures only |

## Deferred Integrations

These are intentionally not part of the base crate:

- SQL storage;
- embedding clients;
- LLM clients;
- HTTP servers;
- MCP servers;
- private deployment scripts.

They may be added later behind explicit Cargo features such as `storage-sql`,
`embedding-http`, or `llm-client`.
