# task.md

## Branch

`codex/ai-sec-runtime-determinism`

## Wave

`Wave 2`, можно запускать параллельно с `04-provider-contract-refactor` и `06-web-target-structure`.

## Mission

Убрать поведенческую случайность и спорную семантику рантайма `ai-sec`.

## Dependencies

- базироваться на `codex/weekend-integration` после merge ветки `02-ai-sec-dx-and-launch`.

## Sources Of Truth

- `Branch_tasks.md`
- `Roadmap_weekend.md`
- `refactoring.md`
- `README.md` только если меняется описание `--limit`

## Allowed Scope

- `src/app/runtime.rs`
- `src/engine/runner.rs`
- `src/payloads/loader.rs`
- `src/engine/session.rs` только если это нужно для честной семантики `--limit`

## Forbidden Scope

- `src/scenarios/*`
- `src/providers/*`
- `src/bin/webapp/*`
- документация, кроме обязательного уточнения `--limit`

## Required Outcomes

- payload loading детерминирован;
- порядок payload-ов стабилен;
- `--limit` имеет честную семантику;
- критичные panic-paths в runtime сведены к минимуму.

## Mandatory Checks

- `cargo check --all-targets`
- `cargo test`
- два последовательных запуска одного сценария дают одинаковый порядок payload-ов

## Handoff

- перечислить изменённые runtime-файлы;
- указать, как теперь работает порядок payload-ов;
- описать финальную семантику `--limit`;
- перечислить тесты и прогоны;
- указать остаточные риски.

## Stop And Escalate If

- для решения задачи требуется вмешательство в scenario/provider contracts;
- честная реализация `--limit` требует расширения scope на другие подсистемы.
