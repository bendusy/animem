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

The maintenance repo should pass profile values into CLI tools or adapters.
Source roots, hostnames, credentials, organization names, and real examples
must not move into public code.
