# AGENTS.md — animem public kernel

给 AI agent 和人类开发者的入口文档。代码里看不出来的事全写这里。

## 这个仓库是什么

`animem` 是 storage-free、network-free 的公开内核。只放中性数据结构、trait 契约、纯算法、synthetic 测试和 CLI。

**红线**：禁止引入 `sqlx`、`tokio`、`reqwest`、`fastembed`、PG、真实路径、provider 端点、业务词表。`sqlx` 仅作为 optional dep 在 `db` feature 下存在（默认关闭）。

## 三仓架构

```
animem (这里)         github.com/bendusy/animem           PUBLIC
  ↑ git rev pin
animem-private        github.com/bendusy/animem-private   private
  └── 消费 public trait，实现 PG/LLM/MCP/deploy
```

## 什么放这里 / 什么放 private

| 类型 | public (这里) | private |
|---|---|---|
| 数据结构 (Slug, Library, Candidate) | ✅ | re-export |
| trait 契约 (DocumentStore, MemoryWriteStore) | ✅ | impl |
| 纯算法 (splitter, validator) | ✅ | — |
| CLI (profile validate, plan, scan, split) | ✅ | — |
| PG 查询/SQL | ❌ | ✅ |
| LLM/embed 调用 | ❌ | ✅ |
| MCP/server 实现 | ❌ | ✅ |
| deploy/systemd | ❌ | ✅ |
| 真实词表/profile/eval | ❌ | ✅ |

## 开发流程

```bash
# 1. 改代码
vim src/xxx.rs

# 2. 跑全部门禁
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
bash scripts/scan-sensitive.sh  # 防敏感泄漏

# 3. 提交 + 推送
git add -A
git commit -m "feat: xxx"
git push

# 4. 同步到 private（见 animem-private/AGENTS.md）
```

## 发布 checklist

- `cargo package --list` 无不该打包的文件
- `scan-sensitive.sh` 通过
- `git diff --check` 无 conflict markers
- commit message 不含私有路径/host/业务词
- GitHub Actions CI 通过

## 模块速查

| 模块 | 内容 |
|---|---|
| domain | Slug/Library/ExperienceType/ContentHash 等 11 类型 |
| validator | has_observation_marker/match_any_secret/sha256_hex |
| registry | LibraryRegistry(6 builtin)/LibraryConfig |
| classification | SourceKind/DirectiveStrength/Severity |
| candidate | Candidate/CandidateKind/CandidateStatus/EvidenceSpan |
| document | DocumentAsset/DocumentSection/DocumentCard |
| extract | ExtractRequest/ExtractResult |
| splitter | split_sections + cjk_headings |
| profile | LocalProfile/PathPrivacy/MaintenancePolicy |
| extension | ExtensionProfile/TokenizerConfig/PromotionPolicy |
| maintenance | MaintenancePlan/ScanPlan |
| store | DocumentStore/MemoryWriteStore/RuntimeConfigProvider/ContextAssembler |
| provenance | ProvenanceEvent/ProvenanceRef |
| schema | SchemaArtifact |
| authority | Authority enum |
| CLI | profile/extension/plan/scan/split |
