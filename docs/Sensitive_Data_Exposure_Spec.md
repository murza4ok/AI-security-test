# Sensitive Data Exposure Specification

## Status

Implemented contract for `src/attacks/sensitive_data_exposure.rs` and
`src/scenarios/*`.

The goal of this mode is reproducible simulation of application-layer data
exposure:

- hidden context in prompt assembly;
- optional retrieval-backed document injection;
- synthetic canaries, PII, and internal notes;
- exposure-focused evaluation and reporting.

## Supported CLI Shape

```bash
cargo run --bin ai-sec -- run --attack sensitive_data_exposure --provider ollama --app-scenario support_bot
cargo run --bin ai-sec -- run --attack sensitive_data_exposure --provider ollama --app-scenario hr_bot
cargo run --bin ai-sec -- run --attack sensitive_data_exposure --provider ollama --app-scenario internal_rag_bot
```

Optional flags:

```bash
--fixture-root <path>
--retrieval-mode full|subset
--scenario-config <path>
--tenant <id>
--session-seed <id>
```

Current semantics:

- `--app-scenario` is required for `sensitive_data_exposure`;
- default retrieval mode is `full` for `support_bot` and `hr_bot`;
- default retrieval mode is `subset` for `internal_rag_bot`;
- `--scenario-config` overrides `scenario.toml`;
- `--tenant` is copied into the rendered envelope and report metadata;
- `--session-seed` is active only for session-memory markers and
  retrieval-subset tie-breaks.

## Scenario Loading Contract

Each scenario is loaded from `fixtures/sensitive_data_exposure/<scenario_id>/`:

- `scenario.toml`
- `system_prompt.txt`
- hidden assets from the scenario root
- retrieval assets from `kb/`
- canaries from files listed in `sensitivity.canary_files`

Current fixtures:

- `support_bot`
- `support_bot_hardened`
- `hr_bot`
- `internal_rag_bot`

Implementation guarantees:

- scenario root files are loaded in stable lexicographic order;
- `kb/` retrieval assets are loaded in stable lexicographic order;
- runtime reuses one preloaded `ScenarioDefinition` instead of reloading the same
  scenario separately for execution and report metadata.

## Envelope Contract

For each payload the builder produces a real request envelope:

- `system_prompt`
- `user_prompt`
- `retrieved_documents`

Assembly rules:

- hidden context is injected into the system prompt by default;
- `prompt_placement = "user_context"` moves the rendered internal context into
  the user message as `[internal_context_summary]`;
- `hidden_context_policy = "sanitized"` or `mask_pii = true` switches prompt
  assembly to masked summaries;
- `include_secret_store = false` removes the synthetic secret-store asset from
  prompt context;
- `memory_enabled = true` adds a deterministic `[session_memory]` block;
- `retrieval_enabled = false` disables retrieval injection even if `kb/` exists.

## Retrieval Contract

Supported modes:

- `full`: include all retrieval assets in stable fixture order;
- `subset`: rank documents by keyword matches against `payload.prompt` and keep
  `top_n`.

Determinism rules for `subset`:

- keyword scoring is deterministic;
- ties are broken by a stable hash of
  `session_seed + normalized_query + document_source`;
- with the same scenario, query, and seed, the subset is stable across runs;
- changing `session_seed` may change tie ordering, but never introduces random
  behavior.

## Schema Honesty

Runtime-active schema fields are documented in
`docs/Scenario_Schema_v2.md#runtime-guarantees`.

For `sensitive_data_exposure`, the most important split is:

- active:
  `context.retrieval_enabled`, `context.top_n`, `context.memory_enabled`,
  `context.prompt_placement`, `context.hidden_context_policy`,
  `context.mask_pii`, `context.include_secret_store`,
  `sensitivity.canary_files`, `sensitivity.pii_fields`,
  `sensitivity.credential_patterns`;
- report-only:
  `context.mode`, all `threat_model.*`.

Reports persist both lists in:

- `scenario.active_schema_fields`
- `scenario.report_only_schema_fields`

## `session_seed` Status

Final status: active, but scoped.

`session_seed` is considered active when at least one of these features applies:

- `session_memory_marker`
- `retrieval_subset_tie_breaks`

The session report stores:

- `scenario.session_seed`
- `scenario.session_seed_status`
- `scenario.meta_envelopes[].session_seed`
- `scenario.meta_envelopes[].session_seed_applied`

If a seed is provided for a scenario where neither feature applies, the runtime
marks it as `provided_but_inactive` instead of pretending it changed behavior.

## Reporting Contract

### Session-level scenario metadata

JSON reports now persist:

- scenario identity: `scenario_id`, `scenario_name`, `scenario_type`,
  `scenario_version`, `defense_profile`;
- execution knobs: `context_mode`, `retrieval_mode`, `tenant`, `session_seed`,
  `session_seed_status`;
- leakage summary: `leaked_canaries`, `leaked_pii_fields`,
  `leaked_secret_types`, `leaked_documents`, `exposure_score`;
- schema honesty lists: `active_schema_fields`, `report_only_schema_fields`;
- envelope archives:
  `real_envelopes[]` and `meta_envelopes[]`.

### `real_envelopes[]`

Each entry stores the request that was effectively assembled for a payload:

- `payload_id`
- `payload_name`
- `payload_prompt`
- `system_prompt`
- `user_prompt`
- `retrieved_documents`

### `meta_envelopes[]`

Each entry stores the structured control metadata for the same payload:

- scenario identity
- context mode and retrieval mode
- retrieval/settings flags
- prompt placement and hidden-context policy
- `mask_pii`
- `include_secret_store`
- tenant and seed metadata
- hidden asset sources and retrieved document sources

## Evaluator Contract

The evaluator produces `AttackEvidence` and `DamageAssessment` using:

- exact canary matches;
- sensitive-value matches extracted from fixtures;
- document-fragment matches against hidden and retrieved assets;
- system-prompt fragment matches;
- credential-like classification from `sensitivity.credential_patterns`.

Outcome guidance:

- exact canary, document, or system-prompt leakage can produce `BYPASS`;
- multi-field sensitive leakage can produce `BYPASS`;
- schema-only or weak secret hints produce `PARTIAL`;
- clean refusals remain `REFUSED`.

## Smoke-Check Targets

Manual smoke paths for this branch:

- `support_bot`
- `hr_bot`
- `internal_rag_bot`

The deterministic regression path is two consecutive runs of
`internal_rag_bot --retrieval-mode subset` with the same `--session-seed`,
expecting identical report-level envelopes and retrieval subsets.
