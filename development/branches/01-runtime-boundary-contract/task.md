# task.md

## Branch

`codex/runtime-boundary-contract`

## Wave

`Wave 1`, строго последовательно, первая рабочая ветка.

## Mission

Зафиксировать архитектурную границу между `ai-sec` и `web_target` как между двумя отдельными runtime-подсистемами и убрать двусмысленность запуска.

## Dependencies

- стартовать от `codex/weekend-integration`;
- не запускать параллельно другие ветки, меняющие запуск и контракт документации.

## Sources Of Truth

- `Branch_tasks.md`
- `Roadmap_weekend.md`
- `TZ.md`
- `README.md`
- `Architecture.md`
- `docs/README.md`
- `docs/Rules.md`

## Allowed Scope

- `README.md`
- `Architecture.md`
- `TZ.md`
- `Cargo.toml` только если это необходимо для фиксации бинарного контракта
- новый документационный файл при необходимости

## Forbidden Scope

- `src/engine/*`
- `src/scenarios/*`
- `src/providers/*`
- `src/bin/webapp/*`
- `src/app/*`, кроме минимального исправления запуска, если без этого нельзя зафиксировать контракт

## Required Outcomes

- документация ясно говорит, что `ai-sec` и `web_target` запускаются отдельно;
- допустимый shared contract layer описан честно;
- двусмысленность бинарного запуска устранена;
- ни один основной документ не описывает проект как связанный монолитный runtime.

## Mandatory Checks

- `cargo run --bin ai-sec -- --help`
- `cargo run --bin web_target -- --help` или `cargo check --all-targets`
- ручная сверка `README.md`, `Architecture.md`, `TZ.md`

## Handoff

- перечислить изменённые файлы;
- перечислить принятые contract-решения;
- указать, что именно изменилось в запуске;
- перечислить команды проверки;
- указать оставшиеся риски.

## Stop And Escalate If

- требуется рефакторинг runtime-кода вне разрешённого scope;
- выясняется, что бинарный контракт нельзя зафиксировать без более широкой правки CLI.
