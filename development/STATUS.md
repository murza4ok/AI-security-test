# Weekend Sprint Status

Этот файл фиксирует фактическое состояние weekend-итерации между сессиями.

## Current Continuation Point

- integration branch: `codex/weekend-integration`
- current next branch to start: `codex/ai-sec-dx-and-launch`
- current next task-pack: `development/branches/02-ai-sec-dx-and-launch/task.md`
- prompts location: `prompts.md`

## Completed Stages

### Step 0. Orchestration Layer

Статус:
- completed

Основание:
- orchestration/task-pack snapshot был зафиксирован в bundle commit `e6f92e5`

Артефакты:
- `development/README.md`
- `development/branches/*/task.md`
- `Branch_tasks.md`
- `Roadmap_weekend.md`
- `TZ.md`
- `refactoring.md`

### 01. Runtime Boundary Contract

Статус:
- completed
- reviewed
- merged into `codex/weekend-integration`

Feature branch:
- `codex/runtime-boundary-contract`

Feature commit:
- `d3a5e32`

Integration merge:
- `78f8145`

Что сделано:
- `README.md`, `Architecture.md`, `TZ.md` синхронизированы с контрактом двух отдельных runtime-контуров;
- поддерживаемый запуск зафиксирован как `cargo run --bin ai-sec -- ...` и `cargo run --bin web_target --`;
- bare `cargo run` объявлен неподдерживаемым пользовательским контрактом;
- shared contract layer ограничен внешним API, shared fixtures/manifests/schema и report metadata;
- прямые in-process связи между `ai-sec` и `web_target` объявлены вне контракта.

Проверки:
- `cargo run --bin ai-sec -- --help`
- `cargo check --all-targets`
- ручная сверка `README.md`, `Architecture.md`, `TZ.md`

Reviewer verdict:
- ready for merge

Residual note:
- `cargo run --bin web_target -- --help` не является help-path для текущего `web_target`;
- допустимая проверка для этой ветки была закрыта через `cargo check --all-targets`.

## Not Started Yet

- `02-ai-sec-dx-and-launch`
- `03-ai-sec-runtime-determinism`
- `04-provider-contract-refactor`
- `05-scenario-contract`
- `06-web-target-structure`
- `07-http-target-client`
- `08-multi-turn-foundation`
- `09-reporting-hardening`
- `10-docs-consistency-sweep`
- `11-integration-smoke`

## Resume Procedure

1. Открой `development/STATUS.md`.
2. Убедись, что текущая база — `codex/weekend-integration`.
3. Не запускай повторно `01-runtime-boundary-contract`: он уже выполнен и влит в integration branch.
4. Создай следующую feature-ветку `codex/ai-sec-dx-and-launch` от `codex/weekend-integration`.
5. Используй промпт из `prompts.md` для `development/branches/02-ai-sec-dx-and-launch/task.md`.
6. После завершения ветки обнови этот файл новым completed block и следующим continuation point.
