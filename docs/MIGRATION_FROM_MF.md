# Migration From `mf`

`animem` is intended to become the public replacement for the old `mf`
maintenance surface, without inheriting private deployments or data.

## Replacement Boundary

`animem` should own:

- public document and memory data models;
- local profile schemas;
- maintenance plans derived from profiles;
- provider configuration contracts;
- storage and runtime interfaces behind explicit features.

Private maintenance code should own:

- real source roots;
- private batch names and redaction maps;
- deployment endpoints;
- credentials and provider keys;
- migration runbooks for private stores.

## Migration Order

1. Move hard-coded local selectors into private profiles.
2. Replace source-path persistence with profile-derived source references.
3. Move provider defaults to environment or private profile values.
4. Add public CLI commands that consume a `MaintenancePlan`.
5. Keep private adapters thin enough that they can be deleted after parity.

The public repository must not import old `mf` fixtures, paths, hostnames, or
git history.
