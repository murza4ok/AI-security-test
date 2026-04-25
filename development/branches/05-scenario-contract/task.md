# task.md

## Branch

`codex/scenario-contract`

## Wave

`Wave 3`, строго после завершения `03-ai-sec-runtime-determinism` и `04-provider-contract-refactor`.

## Mission

Привести scenario subsystem к честному, воспроизводимому и документированному контракту.

## Dependencies

- базироваться на `codex/weekend-integration` после merge веток `03-ai-sec-runtime-determinism` и `04-provider-contract-refactor`.

## Sources Of Truth

- `Branch_tasks.md`
- `Roadmap_weekend.md`
- `TZ.md`
- `refactoring.md`
- `docs/Scenario_Schema_v2.md`
- `docs/Sensitive_Data_Exposure_Spec.md`
- `README.md`

## Allowed Scope

- `src/scenarios/*`
- `src/attacks/sensitive_data_exposure.rs`
- `src/app/scenarios.rs`
- `src/app/runtime.rs` только в части scenario metadata
- `src/engine/session.rs` только в части scenario-level metadata
- `fixtures/sensitive_data_exposure/*`
- `docs/Scenario_Schema_v2.md`
- `docs/Sensitive_Data_Exposure_Spec.md`

## Forbidden Scope

- `src/providers/*`
- `src/bin/webapp/*`
- generic classic attacks

## Required Outcomes

- реальные и ложные поля сценарного schema разграничены;
- `session_seed` либо реально работает, либо честно выведен из активного контракта;
- retrieval subset детерминирован;
- двойная загрузка сценария устранена;
- в отчёте сохраняется реальный envelope и meta envelope;
- docs и implementation совпадают.

## Mandatory Checks

- `cargo check --all-targets`
- `cargo test`
- ручной smoke-check `support_bot`
- ручной smoke-check `hr_bot`
- ручной smoke-check `internal_rag_bot`
- повторный прогон на детерминизм

## Handoff

- перечислить изменённые scenario-файлы и fixtures;
- указать финальный статус `session_seed`;
- описать, какие поля schema признаны рабочими;
- перечислить smoke-check и детерминизм-проверки;
- указать остаточные риски.

## Stop And Escalate If

- требуется менять provider/web_target слой;
- выясняется, что честный scenario contract требует нового отдельного этапа.
