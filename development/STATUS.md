# Weekend Sprint Status

Этот файл фиксирует фактическое состояние weekend-итерации между сессиями.

## Current Continuation Point

- integration branch: `codex/weekend-integration`
- current wave: `Wave 2`
- branches allowed to start now:
  - `codex/ai-sec-runtime-determinism`
  - `codex/provider-contract-refactor`
  - `codex/web-target-structure`
- task-packs to use now:
  - `development/branches/03-ai-sec-runtime-determinism/task.md`
  - `development/branches/04-provider-contract-refactor/task.md`
  - `development/branches/06-web-target-structure/task.md`
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

### 02. AI-Sec DX And Launch

Статус:
- completed
- reviewed
- merged into `codex/weekend-integration`

Feature branch:
- `codex/ai-sec-dx-and-launch`

Feature commit:
- `e45a6fb`

Integration merge:
- `e45a6fb`

Что сделано:
- help, usage и after-help синхронизированы с поддерживаемым запуском `cargo run --bin ai-sec -- ...`;
- README приведён к фактическому CLI-контракту `ai-sec`;
- интерактивный режим больше не падает целиком без настроенного провайдера;
- без настроенного провайдера attack-run пункты скрываются, а review/sessions/explain остаются доступны;
- `--output` документирован честно как single-provider path.

Проверки:
- `cargo check --all-targets`
- `cargo run --bin ai-sec -- --help`
- `cargo run --bin ai-sec -- list`
- `cargo run --bin ai-sec -- help run`
- ручной smoke-check интерактивного режима без настроенного провайдера

Reviewer verdict:
- ready for merge

Residual note:
- live smoke запуска атак из интерактивного меню с реально настроенным провайдером не выполнялся в этом окружении;
- pre-existing warning про `display_name` в `web_target` остаётся вне scope ветки.

## Not Started Yet

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
3. Не запускай повторно `01-runtime-boundary-contract` и `02-ai-sec-dx-and-launch`: они уже выполнены и влиты в integration branch.
4. Для `Wave 2` можно стартовать параллельно `03-ai-sec-runtime-determinism`, `04-provider-contract-refactor` и `06-web-target-structure`.
5. Используй соответствующие `development/branches/*/task.md` и соблюдай их allowed scope.
6. После завершения и merge веток `03`, `04` и `06` обнови этот файл перед переходом к `05-scenario-contract`.
