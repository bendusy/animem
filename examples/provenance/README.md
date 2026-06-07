# Synthetic provenance examples

These fixtures demonstrate public, storage-free provenance shapes. They are
synthetic by construction and contain only opaque references, hashes, labels,
offsets, coarse timestamps, and project/runtime/actor/artifact/reason refs.

## Files

| File | Purpose |
|---|---|
| `candidate-evidence.synthetic.json` | Candidate evidence linked to a document section by opaque reference and offsets. |
| `review-decision-event.synthetic.json` | Review decision event using input and output references only. |
| `schema-artifact-manifest.synthetic.json` | Schema artifact metadata with synthetic hashes only. |

## Safety contract

Do not add local paths, endpoint URLs, raw source text, prompts, transcripts,
credential-shaped values, real names, or deployment details. Use neutral labels
such as `Policy Memo A`, `Project Alpha`, and `Review Note B`.

Validate changes with:

```bash
bash scripts/check-redacted-event-payload examples/provenance/candidate-evidence.synthetic.json
bash scripts/scan-sensitive.sh
```
