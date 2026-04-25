# task.md

## Branch

`codex/provider-contract-refactor`

## Wave

`Wave 2`, можно запускать параллельно с `03-ai-sec-runtime-determinism` и `06-web-target-structure`.

## Mission

Стабилизировать provider layer и убрать опасные и дублирующиеся решения.

## Dependencies

- базироваться на `codex/weekend-integration` после merge ветки `02-ai-sec-dx-and-launch`.

## Sources Of Truth

- `Branch_tasks.md`
- `Roadmap_weekend.md`
- `refactoring.md`
- `README.md`
- `docs/Ollama_Demo_Setup.md`
- `docs/Ollama_Demo_Generator.md`

## Allowed Scope

- `src/app/providers.rs`
- `src/providers/*`
- `src/config/mod.rs` только если это необходимо для provider contract
!!!ВНЕСЕНЫ ИЗМЕНЕНИЯ ПОЛЬЗОВАТЕЛЕМ, ПРОВЕРИТЬ task.md, оповестить пользователя перед выполнением о принятии правок!!!
Решение: да, если необходимо УТОЧНИТЬ(то есть сделать определение строже) help/docs, в этом пункте допустимо расширить scope. 

## Forbidden Scope

- `src/scenarios/*`
- `src/bin/webapp/*`
- `src/reporting/*`
- `src/engine/*`, кроме минимальной адаптации интерфейса

## Required Outcomes

- `--model` работает предсказуемо и безопасно;
- provider factory сокращён и приведён к одному внятному пути;
- подготовлена база для HTTP target client;
- ложные интерфейсы либо удалены, либо доведены до рабочего состояния.

## Mandatory Checks

- `cargo check --all-targets`
- `cargo test`
- ручной smoke-check `check --provider ollama`
- ручной smoke-check `check --provider deepseek`
- проверка поведения `--model`

## Handoff

- перечислить изменённые provider-файлы;
- описать финальную семантику `--model`;
- указать, что именно убрано из дублирования;
- перечислить проверки;
- указать совместимость с будущим HTTP target client.

## Stop And Escalate If

- для завершения задачи требуется менять reporting/scenario/web_target слой;
- provider contract нельзя стабилизировать без выхода за разрешённый scope.
