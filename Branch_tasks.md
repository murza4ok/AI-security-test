# Branch Tasks

## 1. Назначение документа

Этот документ разбивает [Roadmap_weekend.md](./Roadmap_weekend.md) на отдельные циклы разработки по веткам. Он нужен как рабочая постановка задач для поэтапной модульной разработки.

`Roadmap_weekend.md` остаётся верхнеуровневым чек-листом результатов.

Этот файл отвечает на другой вопрос:

- какую ветку создавать;
- что в ней можно менять;
- что менять нельзя;
- какие проверки обязательны;
- какой результат считается достаточным для слияния.

## 2. Зафиксированные допущения

На основе уточнений владельца проекта считаем верными следующие правила:

1. `ai-sec` и `web_target` — отдельные runtime-контура.
2. Между ними допустим общий schema/contract слой, если это интерпретируется как результат “разведки” цели и подготовки атакующего инструмента.
3. Отдельный `.env` для `web_target` не требуется; на текущем этапе допускается единый локальный `.env`.
4. `web_target` должен быть самостоятельным web-приложением с возможностью последующей быстрой миграции на внешний хостинг.
5. Сетевые и канальные разграничители на текущем этапе не моделируются: инструмент видит цель без дополнительных сетевых помех.

## 3. Общие правила работы по веткам

### 3.1. Правило владельца записи

На одной ветке должен быть один пишущий исполнитель.

Допускается:

- 1 агент-писатель на ветку;
- 1 проверяющий после завершения работы.

Не допускается:

- несколько пишущих агентов в одну и ту же ветку;
- параллельная запись в один и тот же модуль из разных веток без явного контракта.

### 3.2. Правило границ

Ветка должна менять только те файлы и каталоги, которые ей явно разрешены.

Если в процессе работы стало ясно, что нужно менять соседний модуль, это не делается “по пути”, а выносится:

- либо в отдельную ветку;
- либо в согласованное расширение scope.

### 3.3. Правило результата

Каждая ветка должна давать один завершённый результат, который можно проверить отдельно.

Нельзя делать ветку формата:

- “немного CLI”;
- “немного scenarios”;
- “немного reporting”;
- “и ещё пара случайных cleanup-ов”.

### 3.4. Правило проверки

Перед слиянием любая ветка обязана пройти:

- `cargo check --all-targets`
- релевантные `cargo test`
- минимум один ручной smoke-check по своему сценарию

### 3.5. Правило документации

Документация считается частью поставки любой ветки.

Минимум, который обязан сделать исполнитель:

- проверить документы в своём scope;
- обновить команды, примеры и ограничения, если они изменились;
- не оставлять в `docs/` и корневых `.md` файлах обещания, которых уже нет в коде.

Если ветка меняет пользовательский контракт:

- команды запуска;
- флаги;
- API;
- структуру сценариев;
- формат отчёта;

то ветка обязана обновить соответствующую документацию в своём же scope.

### 3.6. Правило регулярной актуализации

Документация пересматривается регулярно, а не только в конце итерации.

Обязательное правило:

- каждая feature-ветка проверяет свои документы в `docs/` и корневые документы проекта;
- финальная интеграция включает отдельный docs-sweep;
- устаревшие личные журналы и “временные памятки” не возвращаются в репозиторий.

## 4. Базовая схема ветвления

### Основные ветки процесса

- `main` — стабильная ветка
- `codex/weekend-integration` — интеграционная ветка выходных
- feature-ветки от `codex/weekend-integration`

### Правило слияния

Каждая feature-ветка сливается сначала в `codex/weekend-integration`.

В `main` уходит уже собранная и проверенная интеграционная ветка.

## 5. Параллелизация

### Последовательные блоки

Нужно делать строго последовательно:

1. `codex/runtime-boundary-contract`
2. `codex/ai-sec-dx-and-launch`

### Разрешённая параллель после стабилизации базового контракта

Можно запускать параллельно:

- `codex/ai-sec-runtime-determinism`
- `codex/provider-contract-refactor`
- `codex/web-target-structure`

### Следующая волна

После завершения предыдущей волны:

- `codex/scenario-contract`

После неё:

- `codex/http-target-client`

После неё:

- `codex/multi-turn-foundation`

После неё:

- `codex/reporting-hardening`

После неё:

- `codex/docs-consistency-sweep`

### Финал

После завершения всех feature-веток:

- `codex/integration-smoke`

## 6. Матрица веток

### 6.1. `codex/runtime-boundary-contract`

Цель:

- зафиксировать правильную границу между `ai-sec` и `web_target` как между двумя отдельными runtime-подсистемами.

Разрешённый scope:

- `README.md`
- `Architecture.md`
- `TZ.md`
- `Roadmap_weekend.md` не трогать
- новый/отдельный документационный файл при необходимости
- `Cargo.toml` только если нужно явно зафиксировать модель запуска бинарей

Запрещено менять:

- `src/engine/*`
- `src/scenarios/*`
- `src/providers/*`
- `src/bin/webapp/*`
- `src/app/*`, кроме минимального исправления запуска, если без этого невозможно зафиксировать контракт

Задачи:

- описать и/или закрепить, что `ai-sec` и `web_target` запускаются отдельно;
- определить допустимый shared contract layer;
- убрать двусмысленность в запуске бинарей;
- зафиксировать, что связь подсистем — только через внешний контракт, а не через прямой вызов внутренних модулей.

Обязательные проверки:

- `cargo run --bin ai-sec -- --help`
- `cargo run --bin web_target -- --help` если CLI для web_target отсутствует, достаточно `cargo check --all-targets`
- проверка, что документация не обещает связанный монолитный runtime

Критерий готовности:

- любой новый исполнитель однозначно понимает, что это два отдельных runtime-контура.

Параллелить:

- нет

### 6.2. `codex/ai-sec-dx-and-launch`

Цель:

- привести запуск и пользовательский контракт `ai-sec` в честное состояние.

Разрешённый scope:

- `Cargo.toml`
- `README.md`
- `src/cli/*`
- `src/app/mod.rs`
- `src/app/interactive.rs`
- `src/app/runtime.rs` только в части CLI/DX-поведения

Запрещено менять:

- `src/scenarios/*`
- `src/providers/*`
- `src/bin/webapp/*`
- evaluator logic

Задачи:

- исправить способ запуска CLI;
- обновить примеры команд;
- починить/help/usage/after_help;
- сделать интерактивный режим честным относительно текущих возможностей;
- убрать явные UX-несоответствия между README и реальным поведением.

Обязательные проверки:

- `cargo check --all-targets`
- `cargo run --bin ai-sec -- list`
- `cargo run --bin ai-sec -- help run`
- ручной smoke-check интерактивного режима

Критерий готовности:

- документация и фактический запуск `ai-sec` совпадают.

Параллелить:

- нет, эта ветка должна завершиться до остальных активных модификаций CLI-контракта

### 6.3. `codex/ai-sec-runtime-determinism`

Цель:

- убрать поведенческую случайность и спорную семантику в рантайме `ai-sec`.

Разрешённый scope:

- `src/app/runtime.rs`
- `src/engine/runner.rs`
- `src/payloads/loader.rs`
- при необходимости `src/engine/session.rs` только в части счётчиков, если это потребуется для честной семантики `--limit`

Запрещено менять:

- `src/scenarios/*`
- `src/providers/*`
- `src/bin/webapp/*`
- README, кроме если обнаружится обязательное изменение описания `--limit`

Задачи:

- сделать deterministic payload loading;
- зафиксировать стабильный порядок payload-ов;
- привести `--limit` к честной семантике;
- минимизировать panic-paths в runtime, где это критично для CLI;
- не менять сценарный контракт и HTTP-логику.

Обязательные проверки:

- `cargo check --all-targets`
- `cargo test`
- два последовательных запуска одного и того же сценария дают одинаковый порядок payload-ов

Критерий готовности:

- поведение рантайма не зависит от случайного порядка `read_dir()` и не вводит оператора в заблуждение.

Параллелить:

- да

### 6.4. `codex/provider-contract-refactor`

Цель:

- стабилизировать provider layer и убрать опасные/дублирующиеся решения.

Разрешённый scope:

- `src/app/providers.rs`
- `src/providers/*`
- `src/config/mod.rs` только если это необходимо для корректного provider contract

Запрещено менять:

- `src/scenarios/*`
- `src/bin/webapp/*`
- `src/reporting/*`
- `src/engine/*`, кроме минимальной адаптации импорта/интерфейса

Задачи:

- исправить поведение `--model`;
- сократить дублирование provider factory;
- подготовить почву для отдельного HTTP target client;
- убрать ложные интерфейсы или сделать их реальными, если это относится именно к provider contract.

Обязательные проверки:

- `cargo check --all-targets`
- `cargo test`
- ручной smoke-check:
  - `check --provider ollama`
  - `check --provider deepseek`
  - проверка безопасного поведения `--model`

Критерий готовности:

- provider factory работает предсказуемо, а override модели не ломает мульти-provider сценарии.

Параллелить:

- да

### 6.5. `codex/scenario-contract`

Цель:

- привести scenario subsystem к честному и воспроизводимому состоянию.

Разрешённый scope:

- `src/scenarios/*`
- `src/attacks/sensitive_data_exposure.rs`
- `src/app/scenarios.rs`
- `src/app/runtime.rs` только в части scenario metadata
- `src/engine/session.rs` только в части scenario-level metadata
- `fixtures/sensitive_data_exposure/*`
- `docs/Scenario_Schema_v2.md`
- `docs/Sensitive_Data_Exposure_Spec.md`

Запрещено менять:

- `src/providers/*`
- `src/bin/webapp/*`
- generic classic attacks

Задачи:

- определить реальные и ложные поля сценарного schema;
- довести `session_seed` до рабочего состояния или честно убрать его из активного контракта;
- сделать deterministic retrieval subset;
- убрать двойную загрузку сценария;
- сохранять в отчёте реальный envelope/meta envelope;
- унифицировать session-level scenario metadata;
- выровнять docs и implementation.

Обязательные проверки:

- `cargo check --all-targets`
- `cargo test`
- ручной smoke-check:
  - `support_bot`
  - `hr_bot`
  - `internal_rag_bot`
- сравнение повторных прогонов на детерминизм

Критерий готовности:

- scenario-driven режим воспроизводим, а манифест и код больше не противоречат друг другу.

Параллелить:

- нет, после завершения веток `runtime-determinism` и `provider-contract-refactor`

### 6.6. `codex/web-target-structure`

Цель:

- довести `web_target` до модульной и пригодной к развитию структуры.

Разрешённый scope:

- `src/bin/web_target.rs`
- `src/bin/webapp/*`
- `fixtures/sensitive_data_exposure/support_bot*`
- минимальные docs по web-target

Запрещено менять:

- `src/scenarios/*`
- `src/providers/*`
- `src/engine/*`
- `src/reporting/*`

Задачи:

- явно разделить:
  - auth/session;
  - state/data access;
  - policy;
  - tool-like backend behavior;
  - rendering;
- уменьшить связанность длинной policy-функции;
- стабилизировать API `/api/chat`;
- сохранить самостоятельность `web_target` как приложения, пригодного к ручному тестированию и будущему выносу на хостинг.

Обязательные проверки:

- `cargo check --all-targets`
- ручной запуск `web_target`
- smoke-check:
  - `/health`
  - `/login`
  - `/chat`
  - `/api/chat`

Критерий готовности:

- `web_target` больше не выглядит как временный скрипт, а имеет понятные модули и стабильный API.

Параллелить:

- да

### 6.7. `codex/http-target-client`

Цель:

- научить `ai-sec` атаковать `web_target` как внешнюю HTTP-цель.

Разрешённый scope:

- новый target client / новый модуль target integration
- `src/app/*` в части нового режима запуска
- `src/providers/*` только если решено встраивать HTTP target в provider-подобный слой
- `src/reporting/*` только в части target metadata
- docs по HTTP mode

Запрещено менять:

- внутреннюю policy-логику `web_target`, кроме необходимой фиксации API response contract
- `src/scenarios/*`, если это не связано напрямую с HTTP target metadata

Задачи:

- реализовать HTTP-клиент для `/api/chat`;
- поддержать login/session/cookie persistence;
- добавить CLI-флаги для base URL, user, profile;
- записывать metadata цели в отчёт;
- не связывать `ai-sec` с внутренними Rust-модулями `web_target`.

Обязательные проверки:

- `cargo check --all-targets`
- `cargo test`
- ручной end-to-end smoke:
  - поднять `web_target`
  - выполнить атаку из `ai-sec` через HTTP
  - проверить JSON report

Критерий готовности:

- `ai-sec` атакует `web_target` через внешний контракт, а не через внутренние вызовы.

Параллелить:

- нет, только после `web-target-structure` и `provider-contract-refactor`

### 6.8. `codex/multi-turn-foundation`

Цель:

- заложить основу для атак, зависящих от промежуточных ответов цели.

Разрешённый scope:

- `src/engine/*`
- `src/attacks/*` в части execution model
- payload format / payload docs
- `src/app/runtime.rs` в части запуска цепочек
- `src/reporting/*` только для transcript/chains metadata

Запрещено менять:

- provider factory, если это не требуется для interface adaptation
- внутренности `web_target`, кроме фиксации API-контракта под chain execution

Задачи:

- добавить базовую conversation-chain abstraction;
- определить формат chain payload-ов;
- поддержать 2–5 шагов;
- сохранить transcript;
- подготовить основу для response-aware generative mode.

Обязательные проверки:

- `cargo check --all-targets`
- `cargo test`
- ручной smoke:
  - multi-turn against local flow
  - multi-turn against HTTP target

Критерий готовности:

- проект умеет выполнять и фиксировать не только single-shot атаки.

Параллелить:

- нет, только после HTTP target integration

### 6.9. `codex/reporting-hardening`

Цель:

- завершить reporting model под новые режимы и убрать рассинхронизацию derived metrics.

Разрешённый scope:

- `src/reporting/*`
- `src/engine/session.rs`
- `src/engine/damage.rs`
- `src/engine/evaluator.rs` только если это требуется для согласованности evidence metadata
- docs по reports/review

Запрещено менять:

- provider layer
- web-target policy
- scenario builder logic, кроме required metadata plumbing

Задачи:

- сделать summary/derived metrics с одним источником истины;
- расширить terminal и JSON reports под:
  - scenario envelope metadata;
  - HTTP target metadata;
  - tool decisions;
  - redaction;
  - multi-turn transcripts;
- сохранить удобство review и compare.

Обязательные проверки:

- `cargo check --all-targets`
- `cargo test`
- ручной review:
  - обычная CLI session
  - scenario session
  - HTTP target session
  - multi-turn session

Критерий готовности:

- отчёты достаточно полны для статьи, ручного review и сравнения прогонов.

Параллелить:

- нет, делать после `multi-turn-foundation`

### 6.10. `codex/docs-consistency-sweep`

Цель:

- выровнять живую документацию проекта с фактическим состоянием кода и веточного плана.

Разрешённый scope:

- `README.md`
- `docs/*`
- `Architecture.md`
- `TZ.md`
- `Branch_tasks.md`
- `Test_instruction.md`
- другие `.md` файлы только если они реально участвуют в пользовательском или операторском контуре

Запрещено менять:

- `src/*`
- `fixtures/*`
- `payloads/*`
- `Cargo.toml`, если это не требуется для исправления ошибочной команды в документации

Задачи:

- убрать или архивировать устаревшие документы и дублирующие памятки;
- выровнять команды запуска под реальную бинарную модель;
- убедиться, что `docs/` содержит только живые и поддерживаемые документы;
- проверить, что root-docs и `docs/*` не противоречат друг другу;
- зафиксировать, где находится актуальный план, где находится ТЗ и где находится технический аудит.

Обязательные проверки:

- ручной проход по `README.md`, `docs/*`, `TZ.md`, `Architecture.md`, `Branch_tasks.md`
- проверка, что пользовательские команды в документации соответствуют реальному запуску
- выборочная сверка минимум 3 команд из документации с фактическим CLI

Критерий готовности:

- новый исполнитель понимает, какие документы живые, где искать истину и какие команды реально работают.

Параллелить:

- нет, делать после `reporting-hardening`

### 6.11. `codex/integration-smoke`

Цель:

- собрать всё в одну интеграционную проверку без добавления новых фич.

Разрешённый scope:

- только интеграционные фиксы
- минимальные doc-fixes, найденные на финальном smoke-check

Запрещено менять:

- архитектуру модулей;
- контракты API;
- payload semantics;
- schema без крайней необходимости.

Задачи:

- слить feature-ветки в `codex/weekend-integration`;
- прогнать общую матрицу проверок;
- убедиться, что результат сравним с `Roadmap_weekend.md`;
- зафиксировать список того, что действительно закрыто за итерацию.

Обязательные проверки:

- `cargo check --all-targets`
- `cargo test`
- ручной smoke:
  - запуск `ai-sec`
  - запуск `web_target`
  - scenario-driven run
  - HTTP attack run
  - review/compare

Критерий готовности:

- интеграционная ветка готова к сравнению с `Roadmap_weekend.md` и потенциальному слиянию в `main`.

Параллелить:

- нет

## 7. Минимальная команда проверки для каждой ветки

Каждая ветка должна в финальном отчёте по себе указывать:

- какие файлы были изменены;
- какие документы были проверены и обновлены;
- какие команды были запущены;
- какие тесты прошли;
- какой ручной сценарий был проверен;
- какие риски остались;
- на какой пункт `Roadmap_weekend.md` ветка работает.

## 8. Практическая схема запуска разработки

Правильный старт работ:

1. Создать `codex/weekend-integration`.
2. Сделать `codex/runtime-boundary-contract`.
3. Сделать `codex/ai-sec-dx-and-launch`.
4. Запустить параллельно:
   - `codex/ai-sec-runtime-determinism`
   - `codex/provider-contract-refactor`
   - `codex/web-target-structure`
5. После этого:
   - `codex/scenario-contract`
6. Потом:
   - `codex/http-target-client`
7. Потом:
   - `codex/multi-turn-foundation`
8. Потом:
   - `codex/reporting-hardening`
9. Потом:
   - `codex/docs-consistency-sweep`
10. В конце:
   - `codex/integration-smoke`

## 9. Что считать ошибкой процесса

Следующие ситуации считаются неправильной организацией работы:

- одна ветка меняет сразу `providers`, `scenarios`, `web_target` и `reporting`;
- две параллельные ветки пишут в одни и те же файлы;
- feature-ветка начинает “заодно” решать проблемы другого этапа;
- новая фича добавляется поверх ложного или недоделанного контракта;
- документация откладывается “на потом”, если менялся пользовательский интерфейс.
