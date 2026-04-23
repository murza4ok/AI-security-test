# task.md

## Branch

`codex/multi-turn-foundation`

## Wave

`Wave 5`, строго после `07-http-target-client`.

## Mission

Заложить основу для multi-turn атак, зависящих от промежуточных ответов цели.

## Dependencies

- базироваться на `codex/weekend-integration` после merge ветки `07-http-target-client`.

## Sources Of Truth

- `Branch_tasks.md`
- `Roadmap_weekend.md`
- `TZ.md`
- `README.md`
- документы по payload-формату, если они появятся в пределах ветки

## Allowed Scope

- `src/engine/*`
- `src/attacks/*` в части execution model
- payload format и payload docs
- `src/app/runtime.rs` в части запуска цепочек
- `src/reporting/*` только для transcript и chains metadata

## Forbidden Scope

- provider factory, если это не требуется для адаптации интерфейса
- внутренности `web_target`, кроме фиксации API-контракта под chain execution

## Required Outcomes

- введена базовая conversation-chain abstraction;
- определён формат chain payload-ов;
- поддержаны цепочки из 2–5 шагов;
- сохраняется transcript;
- создана база для response-aware generative mode.

## Mandatory Checks

- `cargo check --all-targets`
- `cargo test`
- ручной smoke-check multi-turn against local flow
- ручной smoke-check multi-turn against HTTP target

## Handoff

- перечислить изменённые engine/attack/reporting файлы;
- описать новый chain format;
- описать, как хранится transcript;
- перечислить проверки;
- указать ограничения первой multi-turn версии.

## Stop And Escalate If

- для завершения задачи требуется переработка provider factory или web-target beyond allowed scope;
- multi-turn logic не удаётся добавить без отдельного контракта в reporting layer.
