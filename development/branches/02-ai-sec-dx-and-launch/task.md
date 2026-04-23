# task.md

## Branch

`codex/ai-sec-dx-and-launch`

## Wave

`Wave 1`, строго после `01-runtime-boundary-contract`.

## Mission

Привести запуск и пользовательский контракт `ai-sec` к честному и предсказуемому состоянию.

## Dependencies

- базироваться на `codex/weekend-integration` после merge ветки `01-runtime-boundary-contract`;
- не запускать параллельно другие ветки, меняющие CLI-контракт.

## Sources Of Truth

- `Branch_tasks.md`
- `Roadmap_weekend.md`
- `README.md`
- `Test_instruction.md`
- `docs/README.md`

## Allowed Scope

- `Cargo.toml`
- `README.md`
- `src/cli/*`
- `src/app/mod.rs`
- `src/app/interactive.rs`
- `src/app/runtime.rs` только в части CLI и DX

## Forbidden Scope

- `src/scenarios/*`
- `src/providers/*`
- `src/bin/webapp/*`
- evaluator logic

## Required Outcomes

- команды запуска `ai-sec` документированы и реально работают;
- `help`, usage и after-help отражают текущие возможности;
- интерактивный режим не обещает того, чего проект не умеет;
- README и фактическое поведение CLI совпадают.

## Mandatory Checks

- `cargo check --all-targets`
- `cargo run --bin ai-sec -- list`
- `cargo run --bin ai-sec -- help run`
- ручной smoke-check интерактивного режима

## Handoff

- перечислить изменённые команды запуска;
- указать, какие документы обновлены;
- перечислить CLI-правки;
- перечислить проверки;
- указать оставшиеся UX-риски.

## Stop And Escalate If

- нужная правка уходит в scenario/provider/web_target логику;
- для честного help требуется переработка вне разрешённого scope.
