# Weekend Sprint Status

Этот файл фиксирует фактическое состояние weekend-итерации между сессиями.

## Current Continuation Point

- integration branch: `codex/weekend-integration`
- current wave: `Wave 2`
- completed in current wave:
  - `codex/web-target-structure`
- blocked before `Wave 3`:
  - `codex/ai-sec-runtime-determinism`
  - `codex/provider-contract-refactor`
- next work before `05-scenario-contract`:
  - resolve remaining report/metadata contract in `03-ai-sec-runtime-determinism`
  - resolve provider-contract scope/diagnostic issues in `04-provider-contract-refactor`
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

### 06. Web-Target Structure

Статус:
- completed
- merged into `codex/weekend-integration`

Feature branch:
- `codex/web-target-structure`

Feature commit:
- `0439b30`

Integration merge:
- `0439b30`

Что сделано:
- `web_target` разделён на bootstrap, handlers, auth, state, policy, tools и html layers;
- `POST /api/chat` сохранён как session-backed JSON endpoint со стабильными полями ответа;
- route handling вынесен из `src/bin/web_target.rs` в модульный `src/bin/webapp/handlers.rs`;
- backend tool-like behavior выделен в `src/bin/webapp/tools.rs`;
- `README.md` и `Architecture.md` дополнены актуальным web-target contract.

Проверки:
- `cargo check --all-targets --offline`
- `cargo test --bin web_target --offline`
- live smoke-check `cargo run --offline --bin web_target --`
- `GET /health` -> `200 OK`
- `GET /login` -> `200 OK`
- `GET /chat` without session -> `303 /login?error=session-required`
- `POST /login` (`customer_alice`, `naive`) -> `303 /chat` + session cookie
- `GET /chat` with session -> `200 OK`
- `POST /api/chat` with session -> `200 OK`

Reviewer note:
- reviewer-agent stalled on procedural re-check after branch commit; integration decision was taken by coordinator based on clean branch state and completed live smoke evidence.

Residual note:
- route-level automated tests for `/api/chat` success/`401`/`400` are still absent;
- live smoke был подтверждён на path `customer_alice + naive`, остальные профили не прогонялись в этой ветке.

## In Progress / Blocked

### 03. AI-Sec Runtime Determinism

Статус:
- implementation present on feature branch
- not ready for merge

Feature branch:
- `codex/ai-sec-runtime-determinism`

Текущее состояние:
- deterministic payload loading, stable file order and honest `--limit` logic реализованы;
- `cargo check --offline --all-targets` и `cargo test --offline` проходят;
- smoke-check через local `ollama` подтвердил стабильный persisted payload order в saved JSON reports;
- reviewer всё ещё считает не до конца закрытой честную семантику session-level generation metadata для mixed multi-attack runs.

Блокер:
- нужно договориться и зафиксировать, что именно должен означать `session.config.generated_variants_per_attack`:
  - configured cap after normalization;
  - или фактически выполненное generated count;
  - или поле должно быть заменено/дополнено другим contract layer.

### 04. Provider Contract Refactor

Статус:
- implementation present on feature branch
- not ready for merge

Feature branch:
- `codex/provider-contract-refactor`

Текущее состояние:
- единый provider factory path и shared OpenAI-compatible layer реализованы;
- `cargo check --offline --all-targets` и `cargo test --offline` проходят;
- найден reviewer finding по регрессии explicit `--provider` diagnostics внутри scope;
- найден contract drift по `--model`, но его честное исправление уже упирается в help/docs вне allowed scope текущего task-pack.

Блокер:
- нужно либо:
  - исправить только provider diagnostics в рамках текущего scope и отдельно решить docs/help;
  - либо явно расширить scope ветки для честного завершения `--model` contract.

## Not Started Yet

- `05-scenario-contract`
- `07-http-target-client`
- `08-multi-turn-foundation`
- `09-reporting-hardening`
- `10-docs-consistency-sweep`
- `11-integration-smoke`

## Resume Procedure

1. Открой `development/STATUS.md`.
2. Убедись, что текущая база — `codex/weekend-integration`.
3. Не запускай повторно `01-runtime-boundary-contract` и `02-ai-sec-dx-and-launch`: они уже выполнены и влиты в integration branch.
4. Не запускай повторно `06-web-target-structure`: он уже завершён и влит в integration branch.
5. Сначала добей blockers веток `03-ai-sec-runtime-determinism` и `04-provider-contract-refactor`.
6. Только после их честного завершения переходи к `05-scenario-contract`.
