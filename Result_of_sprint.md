# Result of Sprint

## 1. Итог спринта

Да, по фактическому status-tracker выполнены все этапы weekend-плана `01`–`11`.

Финальное состояние зафиксировано в:

- `development/STATUS.md`
- integration branch: `codex/weekend-integration`

Итог спринта:

- `ai-sec` и `web_target` приведены к честному раздельному runtime-контракту;
- scenario-driven, HTTP-target и multi-turn контуры доведены до рабочего состояния;
- reporting, review, compare и документация выровнены с реальным кодом;
- финальный integration smoke пройден на общей ветке без новых feature-изменений.

## 2. Общая оценка соответствия ТЗ

Общая оценка: `ТЗ в основном выполнено`.

Статус по верхнему уровню:

| Область ТЗ | Статус | Комментарий |
| --- | --- | --- |
| Разделение `ai-sec` / `web_target` | Выполнено | Отдельные бинарники, отдельные процессы, HTTP-контракт между ними, документация синхронизирована |
| CLI / DX / help-path | Выполнено | `run`, `list`, `check`, `review`, `compare`, `sessions`, `explain`, интерактивный режим, честные help/docs |
| Educational payload corpus | Выполнено | Curated corpus вынесен во внешние TOML, attack family и explainers сохранены |
| Generative mode | Выполнено частично | Seed-based generation работает, metadata пишется в report; полноценная adaptive generation по промежуточным ответам пока только подготовлена архитектурно |
| Scenario-driven режим | Выполнено | `support_bot`, `hr_bot`, `internal_rag_bot`, hardened-вариант, deterministic envelope/report contract |
| HTTP-target attack mode | Выполнено | Login flow, cookie/session persistence, target metadata, атаки через `/api/chat` |
| Multi-turn chains | Выполнено | Chain format `2..5` шагов, transcript, chain scoring/reporting, HTTP и direct path |
| Provider layer | Выполнено | Единый provider path, retry/backoff, provider diagnostics, safe `--model` semantics |
| Reporting / review / compare | Выполнено | Summary/JSON/report metadata выровнены, scenario/HTTP/multi-turn данные видны в review и summary |
| `web_target` как демонстрационная цель | Выполнено частично | Цель рабочая: login/chat/api/profiles/tools; но полный сравнительный demo-matrix по профилям ещё не зафиксирован как отдельный артефакт |
| Synthetic fixtures / evidence / evaluator | Выполнено | Synthetic scenarios, canaries, PII/document evidence, damage model, exposure score |
| Документация и reproducibility | Выполнено | Docs sweep завершён, status-tracking и smoke-flow описаны честно |

## 3. Сопоставление с ключевыми требованиями ТЗ

### 3.1. Продуктовая цель

Соответствие: `выполнено в основном`.

Что достигнуто:

- инструмент запускает curated и generated атаки;
- умеет работать как с provider API, так и с отдельным `web_target` по HTTP;
- демонстрирует data leakage, prompt extraction, tool misuse и сценарные утечки;
- сохраняет воспроизводимые JSON reports и поддерживает review/compare.

Что осталось неполным:

- adaptive generation по промежуточным ответам цели не доведена до полноценного рабочего контура;
- статья-ready demo-matrix в виде отдельного стабильного сценария для всех профилей/моделей ещё можно усилить.

### 3.2. Архитектура двух runtime-подсистем

Соответствие: `выполнено`.

Подтверждение:

- запуск зафиксирован как `cargo run --bin ai-sec -- ...` и `cargo run --bin web_target --`;
- прямые in-process связи между подсистемами убраны из пользовательского и архитектурного контракта;
- общий слой ограничен HTTP/API, synthetic fixtures и report metadata.

### 3.3. `ai-sec`

Соответствие: `выполнено в основном`.

Закрытые пункты:

- payload-driven атаки;
- generative mode;
- scenario-driven режим;
- review / compare / sessions;
- HTTP target mode;
- multi-turn foundation;
- локальные и облачные provider-ы;
- session/evidence/damage reporting.

Частично закрытый пункт:

- “адаптировать полезную нагрузку под поведение цели” реализован как foundation, но не как полноценный adaptive loop генерации.

### 3.4. `web_target`

Соответствие: `выполнено в основном`.

Закрытые пункты:

- локальный web runtime;
- `/health`, `/login`, `/chat`, `/api/chat`, `/logout`;
- demo users: `guest`, `customer_alice`, `customer_bob`, `agent_support`;
- security profiles: `naive`, `segmented`, `guarded`;
- tool-like backend layer;
- HTTP-атака из `ai-sec`.

Ограничения:

- `web_target` всё ещё сидит на фиксированном `127.0.0.1:3000`;
- отдельного полезного `--help` path у `web_target` нет;
- route-level automated tests можно усилить.

### 3.5. Reporting, review, compare

Соответствие: `выполнено`.

В отчётах теперь есть:

- provider/requested model metadata;
- runtime/session metadata;
- generated payload metadata;
- scenario metadata и envelopes;
- evidence и damage assessment;
- exposure score;
- HTTP target metadata, tool decisions, redactions;
- multi-turn transcript и chain-level metadata.

### 3.6. Нефункциональные требования

Соответствие: `выполнено в основном`.

Что подтверждено:

- используются synthetic data и fixtures;
- payload order, retrieval subset и scenario assembly выпрямлены в детерминированный контракт;
- документация приведена к реальному запуску;
- тесты и smoke-check проходят на итоговой интеграционной ветке.

Что ещё можно улучшать:

- увеличить покрытие именно интеграционных HTTP route-тестов;
- убрать фиксированный порт `3000`;
- расширить демонстрационный benchmark по моделям и security profiles.

## 4. Соответствие критериям приёмки из ТЗ

### 4.1. Для `ai-sec`

| Критерий | Статус |
| --- | --- |
| Понятный help и рабочие команды | Выполнено |
| Educational mode на curated corpus | Выполнено |
| Generative mode генерирует из seed | Выполнено |
| `sensitive_data_exposure` работает на нескольких сценариях | Выполнено |
| Отчёты содержат session/scenario/evidence/damage metadata | Выполнено |
| Есть review и compare | Выполнено |
| Есть HTTP-атака на `web_target` | Выполнено |
| Есть базовый multi-turn workflow | Выполнено |

### 4.2. Для `web_target`

| Критерий | Статус |
| --- | --- |
| Макет поднимается локально | Выполнено |
| Есть логин, чат и API | Выполнено |
| Работают `naive`, `segmented`, `guarded` | Выполнено |
| Есть role/tenant-aware backend behavior | Выполнено в основном |
| Профили заметно отличаются при одинаковых атаках | Выполнено частично |
| `ai-sec` атакует макет по HTTP автоматически | Выполнено |

Примечание:

- архитектурно различия профилей реализованы;
- но отдельный финальный артефакт с формальной side-by-side матрицей `naive vs segmented vs guarded` ещё стоит оформить отдельно.

### 4.3. Для демонстрации статьи

| Критерий | Статус |
| --- | --- |
| Можно показать успешную атаку на слабый макет | Выполнено |
| Можно показать, что усиление backend architecture снижает риск | Выполнено частично |
| Демонстрация опирается на synthetic data и reports | Выполнено |
| Сценарий выглядит как атака на систему, а не только на модель | Выполнено |

Частичность здесь связана не с отсутствием механики, а с тем, что следующий логичный шаг — собрать отдельный show-case пакет для статьи: фиксированные сценарии, профили, модели и готовые скриншоты/отчёты.

## 5. Краткий перечень выполненных работ

По факту спринта были выполнены следующие крупные блоки:

1. Выпрямлен runtime boundary contract между `ai-sec` и `web_target`.
2. Исправлены CLI/DX, help-path и интерактивный режим.
3. Доведены deterministic payload loading, `--limit` и report semantics.
4. Перестроен provider layer и исправлена семантика `--model`.
5. Зафиксирован честный scenario contract с envelope/report metadata.
6. Структурирован `web_target` как отдельная целевая подсистема.
7. Реализован HTTP target client с login flow и cookie persistence.
8. Добавлен multi-turn chain foundation для direct и HTTP режимов.
9. Усилен reporting contract и удобство review/compare.
10. Проведён docs consistency sweep и удалены устаревшие/ложные документы.
11. Пройден финальный integration smoke на общей ветке.

## 6. Что можно считать главным результатом спринта

Главный результат не в одной отдельной фиче, а в том, что проект перестал быть набором частично совпадающих контуров.

После спринта репозиторий представляет собой:

- отдельный атакующий CLI;
- отдельную атакуемую HTTP-цель;
- воспроизводимый сценарный и payload-driven стенд;
- систему отчётов, пригодную для ручного review и сравнения прогонов;
- документацию, которая в целом соответствует реальному запуску.

## 7. Основные остаточные ограничения

На конец спринта техдолг не критический, но заметны следующие зоны для следующей итерации:

1. Полноценная adaptive generation по промежуточным ответам цели ещё не реализована.
2. Нужен формальный demo-matrix для статьи:
   `scenario x profile x model x attack`.
3. `web_target` стоит отвязать от фиксированного порта и усилить route-level tests.
4. Можно расширить benchmark/comparison слой под более системный запуск серий прогонов.
5. Стоит оформить готовый “article demo pack”: сценарии, команды, ожидаемые отчёты, скриншоты и narrative.

## 8. Предложения по дальнейшему развитию

### Приоритет P1

- Довести adaptive attack loop:
  generation на основе промежуточного ответа, а не только seed mutation.
- Собрать и зафиксировать article/demo matrix:
  `support_bot`, `support_bot_hardened`, `naive/segmented/guarded`, минимум 2 модели.
- Усилить `web_target` integration tests и убрать жёсткую привязку к `127.0.0.1:3000`.

### Приоритет P2

- Добавить benchmark runner для серийных сравнений моделей и профилей.
- Расширить damage/evidence аналитику под cross-tenant и memory-bleed кейсы.
- Подготовить экспорт результатов в формат, удобный для статьи и презентации.

### Приоритет P3

- Доработать UX вокруг сохранённых сессий и массового review.
- Подготовить более формализованный demo flow для публичного портфолио.

## 9. Финальный вывод

Спринт можно считать успешным.

Проект приведён к состоянию, которое соответствует основной цели ТЗ: это уже не просто CLI для учебных prompt-атак, а связанный исследовательский стенд по `AI Security` с отдельной целью, воспроизводимыми сценариями, HTTP-атаками, multi-turn foundation и рабочим report/review контуром.

При этом для следующей итерации остаётся ясный и ограниченный фронт работ:

- adaptive logic;
- демонстрационная матрица для статьи;
- усиление интеграционных тестов и polish вокруг `web_target`.
