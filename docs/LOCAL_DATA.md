# Local Data Profiles

`animem` keeps local data maintenance outside the public repository.

The public crate may define neutral profile structs, but real values belong in
private files controlled by the operator. Do not commit those files to this
repository.

## Recommended Layout

```text
private-animem-maintenance/
  profiles/
    local.toml          # ignored, real paths and store targets
    example.toml        # tracked only if fully synthetic
  scripts/
    scan
    backfill
    verify
  state/                # ignored
  logs/                 # ignored
```

## Boundary

- public `animem`: schemas, validation, storage-free core primitives;
- private maintenance repo: source roots, batch names, redaction maps, runbooks;
- private data store: documents, caches, embeddings, database dumps.

The maintenance repo should load a private profile, turn it into a
`MaintenancePlan`, and pass only that plan into CLI tools or adapters. Source
roots, hostnames, credentials, organization names, and real examples must not
move into public code.

Default behavior should use `PathPrivacy::StoreRelativePath`: persisted
references look like `source-id:relative/path.ext`, not absolute local paths.

## Extension Profiles

Use `ExtensionProfile` for local rules that would otherwise become hard-coded
Rust:

- `tokenizer.custom_terms` for local terminology;
- `card_rules.organization_terms` for organization detection;
- `card_rules.document_type_patterns` for local document types;
- `promotion.candidate_type_mappings` for mapping extraction candidates into
  local memory types.

The tracked example at `examples/extension-profile.example.json` is fully
synthetic. Real extension profiles should be ignored by git and loaded by the
private adapter at runtime.
