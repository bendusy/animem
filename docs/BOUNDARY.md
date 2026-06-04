# Boundary

`animem` is the public upstream. It owns neutral schemas, validators, pure
document primitives, synthetic examples, and release gates.

## Allowed

- storage-free Rust domain types;
- profile, extension, and provenance schemas with synthetic examples;
- package, dependency, provenance, and redaction documentation;
- tests that use only neutral names such as `Example Org` or `Project Alpha`.

## Not Allowed

- real document titles, excerpts, cards, queries, or eval outputs;
- private hostnames, LAN addresses, filesystem paths, provider names, or model
  defaults;
- deployment runbooks, database dumps, embeddings, caches, or `.env` files;
- code copied from private repositories with private comments or fixtures.

## Release Gate

Run this before publishing or pushing a release candidate:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
bash scripts/test-redacted-event-payload
bash scripts/test-release-gates
bash scripts/scan-sensitive.sh
package_list="$(mktemp "${TMPDIR:-/tmp}/animem-package-list.XXXXXX")"
trap 'rm -f "$package_list"' EXIT
cargo package --allow-dirty --no-verify --list >"$package_list"
bash scripts/check-source-bundle-allowlist "$package_list"
```

The scanner is a gate, not proof of safety. Review commit messages, tag names,
release notes, examples, and documentation for private terms before release.
