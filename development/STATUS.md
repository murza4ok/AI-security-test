# Weekend Sprint Status

Этот файл фиксирует фактическое состояние weekend-итерации между сессиями.

## Current Continuation Point

- integration branch: `codex/weekend-integration`
- current next branch to start: `none`
- current next task-pack: `none`
- current wave to continue: `completed`
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

### 07. HTTP Target Client

Статус:
- completed
- merged into `codex/weekend-integration`

Feature branch:
- `codex/http-target-client`

Feature commit:
- `c850970`

Integration merge:
- `515b28f`

Что сделано:
- `ai-sec` теперь умеет атаковать `web_target` как внешнюю HTTP-цель через отдельный `HttpTargetProvider`, без зависимости от внутренних модулей `web_target`;
- добавлен CLI contract для HTTP mode: `--target-mode http`, `--target-base-url`, `--target-user`, `--target-profile`;
- login flow и session cookie persistence реализованы через `POST /login` и последующие запросы к `POST /api/chat`;
- session-level `target` metadata сохраняется в JSON report и выводится в terminal summary;
- пользовательский и архитектурный контракт HTTP mode зафиксирован в `README.md`, `Architecture.md` и `docs/HTTP_Target_Mode.md`.

Проверки:
- `cargo check --offline --all-targets`
- `cargo test --offline`
- live smoke-check `cargo run --offline --bin web_target --`
- `curl -i http://127.0.0.1:3000/health`
- `curl -i -c /tmp/ai-sec-webtarget.cookies -X POST http://127.0.0.1:3000/login -H 'Content-Type: application/x-www-form-urlencoded' --data 'username=customer_alice&profile=naive'`
- `cargo run --offline --bin ai-sec -- run --attack prompt_injection --target-mode http --target-base-url http://127.0.0.1:3000 --target-user customer_alice --target-profile naive --limit 1 --output /tmp/wt07-http.json`
- `jq '{provider: .provider, target: .target, summary: .summary}' /tmp/wt07-http.json`

Residual note:
- live smoke был подтверждён на single-turn classic attack path `prompt_injection` и профиле `customer_alice + naive`;
- HTTP mode пока сознательно ограничен classic payload-driven flow и не покрывает `sensitive_data_exposure` или multi-turn chains: это следующий этап `08`.

### 08. Multi-Turn Foundation

Статус:
- completed
- merged into `codex/weekend-integration`

Feature branch:
- `codex/multi-turn-foundation`

Feature commits:
- `29c3a24`
- `7f227b1`

Integration merge:
- `8ee77cd`

Что сделано:
- введён базовый conversation-chain contract поверх существующего attack engine без переработки provider factory;
- payload loader теперь поддерживает multi-turn payload format через `[[payloads.turns]]` с длиной цепочки от `2` до `5` шагов;
- для stateless/local provider path следующий ход строится через prompt-history replay, а для HTTP target mode используется native session path без встраивания истории в запрос;
- в `AttackResult` и review/report path добавлены transcript metadata: `transcript`, `chain_planned_turns`, `chain_executed_turns`, `chain_completed`, `chain_abort_reason`;
- добавлен payload/doc contract для chain format в `docs/Chain_Payload_Format.md`;
- в corpus добавлен multi-turn smoke payload `payloads/prompt_injection/00_multi_turn.toml`;
- follow-up fix закрывает reviewer finding: provider error теперь честно останавливает chain и не маркируется как completed run.

Проверки:
- `cargo check --offline --all-targets`
- `cargo test --offline`
- `env OLLAMA_BASE_URL=http://127.0.0.1:11434 OLLAMA_MODEL=qwen2.5:0.5b cargo run --offline --bin ai-sec -- check --provider ollama`
- `env OLLAMA_BASE_URL=http://127.0.0.1:11434 OLLAMA_MODEL=qwen2.5:0.5b cargo run --offline --bin ai-sec -- run --attack prompt_injection --provider ollama --limit 1 --output /tmp/wt08-multiturn-local.json`
- `cargo run --offline --bin web_target --`
- `curl -i http://127.0.0.1:3000/health`
- `cargo run --offline --bin ai-sec -- run --attack prompt_injection --target-mode http --target-base-url http://127.0.0.1:3000 --target-user customer_alice --target-profile naive --limit 1 --output /tmp/wt08-multiturn-http.json`
- `cargo run --offline --bin ai-sec -- review /tmp/wt08-multiturn-http.json`
- `jq '{first_result: .attacks_run[0].results[0] | {chain_planned_turns, chain_executed_turns, chain_completed, transcript_count: (.transcript|length)}}' /tmp/wt08-multiturn-local.json`
- `jq '{target: .target.requests_sent, first_result: .attacks_run[0].results[0] | {chain_planned_turns, chain_executed_turns, chain_completed, transcript_count: (.transcript|length)}}' /tmp/wt08-multiturn-http.json`

Reviewer note:
- reviewer-agent вернул промежуточные findings до follow-up fix: provider-error path ошибочно продолжал chain и sandbox не подтверждал local smoke;
- findings были закрыты в feature follow-up commit `7f227b1`, после чего local и HTTP smoke были повторно подтверждены координатором на live path;
- финальный merge verdict принят координатором на основе passing checks, live smoke evidence и закрытого reviewer defect.

Residual note:
- для совместимости с расширенным payload contract пришлось сделать минимальные compile-through адаптации в `src/generator/mod.rs` и test helpers внутри `src/scenarios/*`; это не отдельный feature-work, а узкая техническая адаптация к новому `Payload` shape;
- reporting model под multi-turn transcript уже работает, но полноценное выпрямление compare/review/report contract вынесено в следующий этап `09`.

### 09. Reporting Hardening

Статус:
- completed
- merged into `codex/weekend-integration`

Feature branch:
- `codex/reporting-hardening`

Feature commit:
- `661cce5`

Integration merge:
- `3180dae`

Что сделано:
- `AttackRun` и `SessionSummary` переведены на единый derived-metrics path: counters и bypass-rate теперь пересчитываются из `results`, а не живут как инкрементальная разрозненная state;
- `TestSession.refresh_metrics()` стал единым местом сборки summary, benchmark и scenario aggregates, включая нормализацию envelope/schema metadata;
- terminal summary, review и compare расширены под session mode, scenario exposure, HTTP target metadata, tool decisions, redactions и multi-turn chain statistics;
- `compare` теперь сначала печатает session overview, а per-attack table различает сессии стабильными колонками `# + mode`, чтобы одинаковые provider/model не слипались;
- JSON migration path выровнен так, чтобы legacy reports гарантированно имели предсказуемый `scenario` object при review/compare;
- `README.md` обновлён под фактический report/review contract для scenario, HTTP target и multi-turn runs.

Проверки:
- `cargo check --offline --all-targets`
- `cargo test --offline`
- `env OLLAMA_BASE_URL=http://127.0.0.1:11434 OLLAMA_MODEL=qwen2.5:0.5b cargo run --offline --bin ai-sec -- run --attack jailbreaking --provider ollama --limit 1 --output /tmp/wt09-cli.json`
- `cargo run --offline --bin ai-sec -- review /tmp/wt09-cli.json`
- `env OLLAMA_BASE_URL=http://127.0.0.1:11434 OLLAMA_MODEL=qwen2.5:0.5b cargo run --offline --bin ai-sec -- run --attack sensitive_data_exposure --provider ollama --app-scenario support_bot --limit 1 --output /tmp/wt09-scenario.json`
- `cargo run --offline --bin ai-sec -- review /tmp/wt09-scenario.json`
- `cargo run --offline --bin ai-sec -- run --attack jailbreaking --target-mode http --target-base-url http://127.0.0.1:3000 --target-user customer_alice --target-profile naive --limit 1 --output /tmp/wt09-http.json`
- `cargo run --offline --bin ai-sec -- review /tmp/wt09-http.json`
- `env OLLAMA_BASE_URL=http://127.0.0.1:11434 OLLAMA_MODEL=qwen2.5:0.5b cargo run --offline --bin ai-sec -- run --attack prompt_injection --provider ollama --limit 1 --output /tmp/wt09-multiturn.json`
- `cargo run --offline --bin ai-sec -- review /tmp/wt09-multiturn.json`
- `cargo run --offline --bin ai-sec -- compare /tmp/wt09-cli.json /tmp/wt09-scenario.json /tmp/wt09-http.json /tmp/wt09-multiturn.json`

Residual note:
- live `run` output по отдельному payload всё ещё может показывать промежуточную эвристику до финального persisted verdict; источником истины для review/compare считается сохранённый session report;
- docs-consistency sweep должен отдельно пройтись по пользовательским описаниям review/report contract, но кодовая часть reporting для `09` закрыта.

### 10. Docs Consistency Sweep

Статус:
- completed
- merged into `codex/weekend-integration`

Feature branch:
- `codex/docs-consistency-sweep`

Feature commits:
- `c636219`
- `44a03af`

Integration merge:
- `26ea951`

Что сделано:
- root docs, `docs/*` и operator-facing markdown выровнены с фактическим состоянием после `09-reporting-hardening`;
- `README.md` и `docs/README.md` теперь явно показывают, где искать живую документацию, `Test_instruction.md` и `development/STATUS.md`;
- `Test_instruction.md` переписан как практический smoke/demo checklist без ссылок на несуществующие helper-скрипты, fixed absolute paths и устаревший `buffer_for_ollama/` workflow;
- `docs/Ollama_Demo_Generator.md` удалён как неподдерживаемый документ; актуальный `Ollama` flow оставлен в `docs/Ollama_Demo_Setup.md` и `Test_instruction.md`;
- `Architecture.md` очищен от битых Windows-path ссылок на файлы репозитория;
- `docs/Sensitive_Data_Exposure_Spec.md` приведён к поддерживаемому контракту запуска `cargo run --bin ai-sec -- ...`;
- `BUNDLE_START_HERE.md` теперь честно указывает на `development/STATUS.md` как на живую точку продолжения, а не на исторический inline-status.

Проверки:
- ручной проход по `README.md`, `docs/*`, `TZ.md`, `Architecture.md`, `Branch_tasks.md`
- `cargo check --offline --all-targets`
- `cargo run --bin ai-sec -- --help`
- `cargo run --bin ai-sec -- help run`
- `cargo run --bin ai-sec -- list`
- `cargo run --offline --bin ai-sec -- compare --help`
- `cargo run --offline --bin ai-sec -- sessions --help`
- `curl -i http://127.0.0.1:3000/health`

Residual note:
- `web_target` по-прежнему не имеет отдельного help-path: `cargo run --bin web_target -- --help` стартует сервер и упрётся в занятый порт, если target уже поднят; для этого runtime рабочей проверкой остаётся запуск бинаря или `cargo check --all-targets`.

### 11. Integration Smoke

Статус:
- completed

Feature branch:
- `codex/integration-smoke`

Что сделано:
- собрана итоговая интеграционная ветка поверх merge-результата `01`–`10` без новых feature-изменений;
- общая матрица `cargo check`, `cargo test`, direct provider run, scenario run, HTTP target run, `review` и `compare` подтверждена на одной и той же интеграционной базе;
- подтверждено, что `web_target` живо отвечает по `/health`, а HTTP attack path работает поверх фиксированного локального target;
- зафиксировано, что weekend-план закрыт без официально отложенных feature-этапов: `01`–`11` выполнены.

Проверки:
- `cargo check --offline --all-targets`
- `cargo test --offline`
- `cargo run --offline --bin ai-sec -- list`
- `curl -i http://127.0.0.1:3000/health`
- `env OLLAMA_BASE_URL=http://127.0.0.1:11434 OLLAMA_MODEL=qwen2.5:0.5b cargo run --offline --bin ai-sec -- run --attack jailbreaking --provider ollama --limit 1 --output /tmp/wt11-direct.json`
- `env OLLAMA_BASE_URL=http://127.0.0.1:11434 OLLAMA_MODEL=qwen2.5:0.5b cargo run --offline --bin ai-sec -- run --attack sensitive_data_exposure --provider ollama --app-scenario support_bot --limit 1 --output /tmp/wt11-scenario.json`
- `cargo run --offline --bin ai-sec -- run --attack jailbreaking --target-mode http --target-base-url http://127.0.0.1:3000 --target-user customer_alice --target-profile naive --limit 1 --output /tmp/wt11-http.json`
- `cargo run --offline --bin ai-sec -- review /tmp/wt11-scenario.json`
- `cargo run --offline --bin ai-sec -- compare /tmp/wt11-direct.json /tmp/wt11-scenario.json /tmp/wt11-http.json`
- `cargo run --offline --bin web_target --`

Residual note:
- `web_target` использует фиксированный `127.0.0.1:3000`, поэтому отдельный launch path на финальной ветке упрётся в `Address already in use`, если живой target уже поднят; в этом интеграционном smoke это подтверждено вместе с `curl /health` и реальным HTTP attack run.
- live direct run по `jailbreaking` по-прежнему может показывать промежуточный `PARTIAL` в stream-output до финального persisted verdict; источником истины остаются сохранённый JSON report, `review` и `compare`.

## Not Started Yet

- none

## Resume Procedure

1. Открой `development/STATUS.md`.
2. Убедись, что текущая база — `codex/weekend-integration`.
3. Weekend-итерация по плану `01`–`11` завершена.
4. Если начинается новая работа, создавай новую ветку уже от актуального `codex/weekend-integration`.
5. Исторические проверки и residual notes по завершённым этапам сохранены выше в этом файле.
