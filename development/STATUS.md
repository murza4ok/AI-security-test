# Weekend Sprint Status

Этот файл фиксирует фактическое состояние weekend-итерации между сессиями.

## Current Continuation Point

- integration branch: `codex/weekend-integration`
- current next branch to start: `codex/http-target-client`
- current next task-pack: `development/branches/07-http-target-client/task.md`
- current wave to continue: `Wave 4`
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

### 03. AI-Sec Runtime Determinism

Статус:
- completed
- reviewed
- merged into `codex/weekend-integration`

Feature branch:
- `codex/ai-sec-runtime-determinism`

Feature commit:
- `c705ce6`

Integration merge:
- `95f9195`

Что сделано:
- deterministic payload loading, stable file order and honest `--limit` logic реализованы;
- stable seed selection for generated mode зафиксирован без случайного выбора;
- `session.config.generated_variants_per_attack` закреплён как normalized configured target per attack;
- фактическое число выполненных generated payload-ов остаётся source of truth на уровне `attacks_run[].generated_payloads` и `summary.total_generated_payloads`;
- panic-path в runtime display callback убран.

Проверки:
- `cargo check --offline --all-targets`
- `cargo test --offline`
- `env OLLAMA_BASE_URL=http://127.0.0.1:11434 OLLAMA_MODEL=qwen2.5:0.5b cargo run --offline --bin ai-sec -- check --provider ollama`
- два последовательных saved JSON reports с одинаковым persisted payload order
- reviewer verdict: merge-ready

Residual note:
- детерминизм гарантирован на уровне persisted session/report order, а не порядка прихода stdout lines при конкурентном выполнении;
- верификация generated/scenario behavior с полноценно отвечающим provider path в review была ограничена окружением.

### 04. Provider Contract Refactor

Статус:
- completed
- reviewed
- merged into `codex/weekend-integration`

Feature branch:
- `codex/provider-contract-refactor`

Feature commit:
- `cf7c6b3`

Integration merge:
- `3d52c53`

Что сделано:
- единый provider factory path и shared OpenAI-compatible layer реализованы;
- explicit `--provider` path снова отдаёт provider-specific diagnostics;
- `--model` теперь ведёт себя предсказуемо и безопасно:
  - разрешён для single-provider runs;
  - разрешён вместе с explicit `--provider`;
  - отклоняется ранним guardrail при multi-provider config без `--provider`;
- help/docs уточнены по `--model` в рамках пользовательского расширения scope;
- общий OpenAI-compatible provider layer вынесен в `src/providers/openai_compatible.rs`.

Проверки:
- `cargo check --offline --all-targets`
- `cargo test --offline`
- `cargo run --offline --bin ai-sec -- check --provider ollama`
- `cargo run --offline --bin ai-sec -- check --provider deepseek`
- `env OPENAI_API_KEY=dummy OPENAI_MODEL=gpt-4o OLLAMA_MODEL=llama3 cargo run --offline --bin ai-sec -- run --attack jailbreaking --model gpt-4.1-mini --limit 1`
- `env OLLAMA_BASE_URL=http://127.0.0.1:11434 OLLAMA_MODEL=qwen2.5:0.5b cargo run --offline --bin ai-sec -- check --provider ollama`
- reviewer verdict: ready for merge

Residual note:
- пользователь явно разрешил расширение scope для уточнения help/docs по `--model`; это решение сохранено в task-pack `04`.

### 05. Scenario Contract

Статус:
- completed
- merged into `codex/weekend-integration`

Feature branch:
- `codex/scenario-contract`

Feature commit:
- `91b7b78`

Integration merge:
- `91b7b78`

Что сделано:
- scenario definition теперь предзагружается в `ScenarioRunConfig`, а runtime/attack path используют cached definition вместо повторной загрузки с диска;
- retrieval subset сохраняет детерминированный порядок и использует `session_seed` только как явный deterministic tie-break input;
- `session_seed` зафиксирован честно: он влияет на session-memory marker и subset retrieval tie-breaks, а его итоговый статус сериализуется в report metadata;
- в session report теперь сохраняются `real_envelopes` и `meta_envelopes` для scenario runs;
- scenario schema разделён на runtime-active поля и report-only поля, и эта граница отражена и в JSON report metadata, и в docs;
- `mask_pii` перестал быть ложным флагом и теперь действительно форсирует masked/summarized rendering hidden context.

Проверки:
- `cargo check --offline --all-targets`
- `cargo test --offline`
- `env OLLAMA_BASE_URL=http://127.0.0.1:11434 OLLAMA_MODEL=qwen2.5:0.5b cargo run --offline --bin ai-sec -- check --provider ollama`
- ручной smoke-check `support_bot` с JSON report path
- ручной smoke-check `hr_bot` с JSON report path
- ручной smoke-check `internal_rag_bot` с JSON report path
- повторный `internal_rag_bot` run с совпадающими `.scenario.real_envelopes` и `.scenario.meta_envelopes`

Reviewer note:
- reviewer-agent не вернул финальный verdict в срок; merge decision принята координатором на основе clean scope, passing tests и ручной верификации JSON envelope contract.

Residual note:
- в этом окружении scenario runs доходили до report path, но provider completion внутри `ai-sec run ... --provider ollama` завершался `Provider is not configured (missing API key or URL)` несмотря на успешный `check --provider ollama`; это выглядит как существующий provider/runtime path gap вне scope `05`, а не как поломка scenario contract.

## Not Started Yet

- `07-http-target-client`
- `08-multi-turn-foundation`
- `09-reporting-hardening`
- `10-docs-consistency-sweep`
- `11-integration-smoke`

## Resume Procedure

1. Открой `development/STATUS.md`.
2. Убедись, что текущая база — `codex/weekend-integration`.
3. Не запускай повторно `01-runtime-boundary-contract`, `02-ai-sec-dx-and-launch`, `03-ai-sec-runtime-determinism`, `04-provider-contract-refactor`, `05-scenario-contract` и `06-web-target-structure`: они уже завершены и влиты в integration branch.
4. Следующая рабочая ветка по плану: `codex/http-target-client`.
5. Используй task-pack `development/branches/07-http-target-client/task.md`.
6. Перед стартом `07` учитывай residual note из `05`: scenario report contract уже стабилизирован, но provider completion path для local Ollama ещё требует отдельной проверки в рамках следующей волны только если это затронет HTTP mode.
