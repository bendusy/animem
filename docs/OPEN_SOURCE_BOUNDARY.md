# Open Source Boundary

`animem` is maintained as a clean public project. It must not share git history
or operational files with private deployments.

## Allowed

- generic document and memory domain types;
- synthetic tests and examples;
- feature-gated integration interfaces;
- public architecture notes that do not mention private hosts, paths, users, or
  source document contents.

## Not Allowed

- real document cards or extracted snippets;
- organization-specific dictionaries copied from private archives;
- local hostnames, LAN addresses, usernames, or filesystem paths;
- database dumps, embeddings, caches, or benchmark outputs from private data;
- private agent handoff files or runbooks;
- credentials, tokens, API keys, or `.env` files.

## Maintenance Rule

If a feature starts in a private repository, port it into `animem` by rewriting
the public abstraction and tests from scratch. Do not copy fixtures, comments,
or examples that were derived from private documents.

## Private Extension Rule

Domain tuning must cross the boundary as data, not as public Rust defaults.
Allowed public surface:

- profile structs and validators;
- neutral rule-pack schemas;
- synthetic examples;
- adapter traits behind features.

Private-only surface:

- tokenizer dictionaries from real archives;
- organization or person name lists;
- prompt packs tuned on private material;
- evaluation queries derived from private work;
- provider/model defaults and network endpoints.
