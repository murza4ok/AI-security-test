# Refactoring Audit

Аудит сделан после повторного прохода по проекту "с нуля": прочитан `README.md`, просмотрены точки входа (`main`, `cli`, `app`), доменные модули (`attacks`, `engine`, `scenarios`, `providers`, `reporting`) и новый `web_target`. Дополнительно проверены:

- `cargo check --all-targets`
- фактический запуск из README (`cargo run -- list`)
- предупреждения компилятора и места с `#[allow(dead_code)]`

## Коротко

- После появления второго бинаря README уже не соответствует реальному способу запуска.
- Самый большой разрыв между декларацией и реализацией сейчас в scenario-подсистеме: часть полей манифеста и CLI-флагов загружается, но не влияет на поведение.
- В коде много копипасты: фабрика провайдеров, реализации OpenAI/DeepSeek, classic attack family, циклы запуска провайдеров.
- Есть недоведённые до конца функции/фичи: интерактивная настройка провайдера, `verbose`, `system_prompt`, `session_seed`.
- Мёртвый код уже начал маскироваться `#[allow(dead_code)]`, вместо того чтобы удаляться или доделываться.

## Приоритетные проблемы

### P1

- `README` сломан относительно текущего проекта.
  `README.md:31-35` и остальные примеры предлагают `cargo run -- ...`, но после появления `src/bin/web_target.rs` это больше не работает: `cargo run -- list` реально падает с ошибкой выбора бинаря. Нужен `default-run = "ai-sec"` в `Cargo.toml` или полное обновление всех примеров на `cargo run --bin ai-sec -- ...`.

- `--model` опасно применяется ко всем провайдерам сразу.
  `src/app/providers.rs:13-95` переопределяет `model` для каждого настроенного провайдера, а `src/app/runtime.rs:300-301` зовёт это даже когда пользователь не ограничил запуск одним `--provider`. В итоге строка вроде `--model gpt-4o` уходит и в Anthropic, и в Ollama, и в YandexGPT, где это имя модели просто невалидно.

- Сценарные отчёты не сохраняют реальный запрос, который был отправлен модели.
  В `src/attacks/sensitive_data_exposure.rs:95-100` строится полноценный `ScenarioEnvelope`, но в `src/attacks/sensitive_data_exposure.rs:143-147` в отчёт пишется только исходный `payload.prompt`. Это ломает воспроизводимость review: из JSON невозможно восстановить фактический `system_prompt`, injected hidden context и `user_context`-режим.

- Scenario schema обещает больше, чем реально реализовано.
  В `src/scenarios/types.rs:25-78` описаны `mode`, `mask_pii`, `credential_patterns`, `threat_model`, а также richer manifest в целом. По факту рантайм использует только часть полей, причём `credential_patterns` и весь `threat_model` нигде не участвуют в логике, а `mask_pii` не управляет редактированием как самостоятельный флаг. Это опасно не как "лишний JSON", а как ложное ощущение, что сценарий управляется манифестом, хотя часть поведения захардкожена в Rust.

- `--session-seed` фактически не делает сценарий детерминированным.
  README описывает seed как механизм детерминированной сборки сценария, но по коду он используется только как текст внутри memory block (`src/scenarios/builder.rs`, вызов идёт из `src/app/scenarios.rs:38-39`). Ни выбор документов, ни порядок payload-ов, ни генерация seed-ом не управляются. Флаг выглядит реализованным, но по сути является косметическим.

- Порядок payload-ов недетерминирован.
  `src/payloads/loader.rs:119-139` читает `read_dir()` без сортировки. Для инструмента, который позиционируется как benchmark/review CLI, это плохое решение: `--limit`, baseline-прогоны и даже набор seed-пейлоадов в generated-режиме могут отличаться между машинами и запусками.

### P2

- Провайдерная фабрика сильно продублирована.
  `src/app/providers.rs:13-95` и `src/app/providers.rs:97-195` делают почти одно и то же двумя разными путями. Это уже породило побочные эффекты вроде глобального `--model` и разнесённой по двум функциям логики ошибок.

- `OpenAIProvider` и `DeepSeekProvider` почти одинаковые файлы.
  Сравнение `src/providers/openai.rs:19-140` и `src/providers/deepseek.rs:17-140` показывает почти полную дубликацию request/response shape, retry-логики и `complete()`. Отличаются в основном `id`, `display_name` и base URL. Это просится в один OpenAI-compatible provider.

- Большинство attack family оформлены как одинаковые тонкие обёртки.
  `src/attacks/prompt_injection.rs`, `jailbreaking.rs`, `extraction.rs`, `goal_hijacking.rs`, `many_shot.rs`, `context_manipulation.rs`, `token_attacks.rs` повторяют один и тот же шаблон: `id/name/description/resources/load_payloads/execute -> run_classic_payloads`. Для семи файлов это уже не "нормальный шаблон", а таблица данных, которую стоило вынести в декларативный слой.

- Registry каждый раз пересоздаёт все attack-объекты.
  `src/attacks/registry.rs:19-42` строит новый `Vec<Arc<dyn Attack>>` на каждый `all_attacks()` и `find_attack()`. Это мелочь по CPU, но хороший симптом: даже простая справочная структура не кэшируется и не централизуется.

- Runtime-логика запуска продублирована, а сценарий загружается дважды.
  `src/app/runtime.rs:53-80` и `src/app/runtime.rs:344-367` повторяют цикл "запустить по провайдерам -> сохранить отчёт". Плюс `src/app/runtime.rs:189-200` заново вызывает `load_scenario()`, хотя `src/attacks/sensitive_data_exposure.rs:78-82` уже загрузил тот же сценарий для фактического выполнения. Это лишний I/O и разнесённая ответственность за scenario metadata.

- Derived metrics имеют два источника истины.
  В `src/engine/session.rs:70-118` и `src/engine/session.rs:204-270` часть метрик инкрементируется вручную в `add_run()`, а часть пересчитывается в `refresh_metrics()`. При этом `refresh_metrics()` не пересобирает summary полностью из `attacks_run`, а только обновляет отдельные поля. Такая модель легко начинает дрейфовать после миграций отчётов или будущих изменений структуры.

- Retrieval subset возвращает документы даже при нулевой релевантности.
  `src/scenarios/retrieval.rs:18-44` сортирует все документы по score и всегда берёт `top_n.max(1)`. Если запрос не совпал ни с одним ключевым словом, в prompt всё равно попадут "лучшие из нулевых", то есть просто произвольные внутренние документы.

- Scenario loader вручную парсит CSV, хотя в проекте уже есть `csv` crate и второй, нормальный CSV loader.
  `src/scenarios/loader.rs:148-171` режет строки по `split(',')`, что ломается на quoted fields и расходится с `src/bin/webapp/state.rs:234-249`, где уже используется `csv::Reader`. Это и баг, и дублирование одной и той же инфраструктурной задачи.

- Интерактивный режим недоделан.
  `src/app/interactive.rs:16-24` требует хотя бы один настроенный провайдер ещё до показа меню, хотя пункты "Browse Saved Sessions" и "Educational Mode" не нуждаются в провайдере вообще. Пункт `Configure Provider (edit .env)` реализован как printout (`src/app/interactive.rs:65-71`), а `src/cli/menu.rs:52-79` хранит неиспользуемые helper-ы `select_provider()` и `confirm()`.

- `web_target` собран как один большой if/else policy-router.
  `src/bin/webapp/policy.rs` — длинная цепочка substring-checks и ранних `return`. Для демо это терпимо, но сопровождать такие правила тяжело: логика профилей, redaction и customer scoping смешаны в одной функции.

### P3

- `verbose` не используется вообще.
  Флаг парсится в `src/cli/args.rs`, прокидывается через `src/app/mod.rs:11-20`, но дальше нигде не влияет ни на `tracing`, ни на вывод, ни на уровни логирования.

- `AttackConfig.system_prompt` выглядит как feature, но живёт вечно `None`.
  Поле объявлено в `src/attacks/mod.rs:89-117`, используется classic runner-ом, но ни CLI, ни интерактивный режим его не заполняют. Сейчас это мёртвая возможность, а не настоящая фича.

- `LLMProvider::supports_system_prompt()` — мёртвая абстракция с ложным контрактом.
  В `src/providers/traits.rs:85-93` документация обещает fallback: если провайдер не поддерживает system prompt, он будет prepend-нут к user message. Но такого wrapper-а в коде нет, и сам метод нигде не вызывается. Это особенно плохо потому, что комментарий обещает поведение, которого нет.

- `LLMResponse.latency_ms` не используется и уже замаскирован `#[allow(dead_code)]`.
  Поле объявлено в `src/providers/traits.rs:29-42`, но раннеры пересчитывают latency самостоятельно. Это типичный кандидат либо на удаление, либо на реальное использование.

- `RequestSettings` зачем-то помечен `#[allow(dead_code)]`.
  `src/config/mod.rs:67-85`. По факту структура используется. Это уже не suppression "на будущее", а мусорный атрибут, который снижает доверие к предупреждениям компилятора.

- `UserRecord.display_name` не используется.
  `cargo check --all-targets` дал прямое предупреждение на `src/bin/webapp/state.rs:67-73`. Сейчас это просто лишнее поле в модели и демо-данных.

- В проекте есть необработанные panic-paths.
  Например, `src/app/runtime.rs:105` (`lock().unwrap()`), `src/attacks/classic.rs:30` и `src/attacks/sensitive_data_exposure.rs:94` (`expect("semaphore closed")`), `src/providers/mod.rs` (`expect("failed to build HTTP client")`). Для CLI это не всегда критично, но это плохой сигнал качества ошибок.

- `--limit` семантически не совпадает с ожидаемым смыслом.
  `src/engine/runner.rs:40-57` сначала truncates curated payload-ы, а потом append-ит generated payload-ы. То есть `--limit 3 --generated 3` даёт до 6 результатов, хотя флаг читается как общий лимит на категорию.

## Подвисшие и недоведённые до конца вещи

- Интерактивная "настройка провайдера" есть в меню, но не реализована.
- `select_provider()` и `confirm()` существуют, но никто их не зовёт.
- `verbose` существует, но ничего не меняет.
- `system_prompt` как часть `AttackConfig` задуман, но пользователю недоступен.
- `supports_system_prompt()` задуман как часть общего контракта, но не интегрирован.
- `session_seed` задуман как детерминизатор сценария, но реальной детерминизации не делает.

## Дублирующийся функционал

- Сборка провайдеров: `src/app/providers.rs`
- OpenAI-compatible провайдеры: `src/providers/openai.rs` и `src/providers/deepseek.rs`
- Classic attack family: почти все файлы в `src/attacks/`, кроме `classic.rs` и `sensitive_data_exposure.rs`
- Цикл запуска по провайдерам и сохранения отчётов: `src/app/runtime.rs`
- CSV ingestion: `src/scenarios/loader.rs` и `src/bin/webapp/state.rs`
- Guardrail-логика сценариев размазана между fixture-файлами (`system_prompt.txt`) и `src/scenarios/builder.rs`, то есть политика меняется сразу в двух слоях

## Неиспользуемые функции, поля и suppressions

- `src/cli/menu.rs`: `select_provider()`, `confirm()`
- `src/app/mod.rs`: `verbose`
- `src/attacks/mod.rs`: `AttackConfig.system_prompt` фактически не заполняется
- `src/providers/traits.rs`: `supports_system_prompt()`, `LLMResponse.latency_ms`
- `src/bin/webapp/state.rs`: `UserRecord.display_name`
- `src/config/mod.rs`: `#[allow(dead_code)]` на `RequestSettings`
- `src/cli/menu.rs`: `#![allow(dead_code)]` на весь файл

## Что бы я делал в каком порядке

1. Починить DX и документацию.
   Добавить `default-run = "ai-sec"` или переписать README на `cargo run --bin ai-sec -- ...`.

2. Убрать ложные/недоведённые интерфейсы.
   Либо удалить `verbose`, `supports_system_prompt`, `system_prompt`, `session_seed`, лишние manifest-поля, либо довести их до настоящей рабочей логики.

3. Стабилизировать воспроизводимость.
   Сортировать payload-файлы, определить честную семантику `--limit`, перестать терять scenario envelope в JSON-отчётах.

4. Сжать копипасту.
   Сначала провайдеры (`OpenAI-compatible`), затем фабрику провайдеров, затем classic attack family.

5. Пересобрать scenario subsystem вокруг одного источника истины.
   Сейчас часть правил живёт в manifest, часть в builder, часть в evaluator, часть в README. Нужна одна декларативная модель и минимальное количество строковых `if value == "..."`.

6. Удалить мёртвый код и снять suppressions.
   Только после этого предупреждения компилятора снова станут полезным сигналом.
