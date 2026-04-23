# task.md

## Branch

`codex/integration-smoke`

## Wave

`Wave 8`, строго после завершения всех feature-веток.

## Mission

Собрать все результаты в одну интеграционную проверку без добавления новых фич.

## Dependencies

- базироваться на `codex/weekend-integration` после merge веток `01`–`10`;
- до начала ветки все feature-ветки должны быть либо смержены, либо официально отложены.

## Sources Of Truth

- `Branch_tasks.md`
- `Roadmap_weekend.md`
- `TZ.md`
- `README.md`
- `Architecture.md`
- `docs/*`

## Allowed Scope

- только интеграционные фиксы
- минимальные doc-fixes, найденные на финальном smoke-check

## Forbidden Scope

- архитектура модулей
- контракты API
- payload semantics
- schema без крайней необходимости

## Required Outcomes

- feature-ветки собраны в `codex/weekend-integration`;
- пройдена общая матрица проверок;
- фактический результат сравним с `Roadmap_weekend.md`;
- зафиксировано, что реально закрыто за weekend-итерацию.

## Mandatory Checks

- `cargo check --all-targets`
- `cargo test`
- ручной smoke запуск `ai-sec`
- ручной smoke запуск `web_target`
- ручной smoke scenario-driven run
- ручной smoke HTTP attack run
- ручной smoke review/compare

## Handoff

- перечислить интеграционные фиксы;
- перечислить итоговые проверки;
- указать статус каждого крупного этапа roadmap;
- указать, что готово к merge в `main`, а что остаётся на следующую итерацию.

## Stop And Escalate If

- интеграционная ветка требует новых feature-change вместо интеграционного фикса;
- обнаружен конфликт контрактов между уже смерженными ветками.
