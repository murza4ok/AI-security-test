# task.md

## Branch

`codex/weekend-integration`

## Role

Это не writer-ветка. Это управляющая и интеграционная ветка weekend-итерации.

## Mission

Поддерживать чистую сборочную базу для всех feature-веток и контролировать порядок merge.

## Owner

- координатор процесса;
- интеграционный владелец;
- не запускать на эту ветку несколько writer-agent.

## Sources Of Truth

- `development/README.md`
- `Branch_tasks.md`
- `Roadmap_weekend.md`
- `TZ.md`
- `refactoring.md`

## Responsibilities

- создать ветку `codex/weekend-integration` от актуальной рабочей базы;
- запускать feature-ветки только от неё;
- принимать в merge только ветки с завершённым handoff;
- не смешивать в интеграционной ветке незавершённые feature-change;
- вести порядок слияния по волнам из `development/README.md`.

## Allowed Actions

- создание feature-веток;
- merge готовых feature-веток;
- минимальная синхронизация task-pack и рабочего порядка, если это требуется для orchestration;
- финальный integration smoke-check.

## Forbidden Actions

- использовать ветку как место хаотичной разработки;
- запускать параллельных writer-agent прямо в `codex/weekend-integration`;
- вносить крупные feature-изменения напрямую без отдельной feature-ветки.

## Merge Gate For Incoming Branches

- есть handoff по ветке;
- пройдены обязательные команды;
- выполнен ручной smoke-check;
- документация в scope ветки обновлена;
- reviewer или координатор не нашли нарушений scope.

## Exit Condition

- все целевые feature-ветки либо смержены, либо официально отложены;
- ветка готова к `11-integration-smoke`;
- после финальной проверки можно принимать решение о merge в `main`.
