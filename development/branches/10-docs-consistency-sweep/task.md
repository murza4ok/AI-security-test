# task.md

## Branch

`codex/docs-consistency-sweep`

## Wave

`Wave 7`, строго после `09-reporting-hardening`.

## Mission

Финально выровнять живую документацию проекта с фактическим состоянием кода и веточного плана.

## Dependencies

- базироваться на `codex/weekend-integration` после merge ветки `09-reporting-hardening`.

## Sources Of Truth

- `Branch_tasks.md`
- `Roadmap_weekend.md`
- `README.md`
- `docs/*`
- `Architecture.md`
- `TZ.md`
- `Test_instruction.md`
- `refactoring.md`

## Allowed Scope

- `README.md`
- `docs/*`
- `Architecture.md`
- `TZ.md`
- `Branch_tasks.md`
- `Test_instruction.md`
- другие `.md` файлы только если они реально участвуют в пользовательском или операторском контуре

## Forbidden Scope

- `src/*`
- `fixtures/*`
- `payloads/*`
- `Cargo.toml`, кроме случая, когда иначе нельзя исправить документированную команду

## Required Outcomes

- в проекте не остаётся устаревших поддерживаемых документов;
- команды запуска и примеры совпадают с реальностью;
- root-docs и `docs/*` не противоречат друг другу;
- новый исполнитель понимает, где находится актуальная документация.

## Mandatory Checks

- ручной проход по `README.md`, `docs/*`, `TZ.md`, `Architecture.md`, `Branch_tasks.md`
- выборочная сверка минимум 3 команд из документации с реальным CLI
- сверка ссылок на roadmap, ТЗ и технический аудит

## Handoff

- перечислить обновлённые и удалённые документы;
- указать, какие команды были перепроверены руками;
- перечислить найденные и устранённые противоречия;
- указать, остались ли документы, требующие отдельного этапа.

## Stop And Escalate If

- документация расходится с кодом из-за незавершённой предыдущей ветки;
- для правки документации требуется фактический кодовый фикс вне allowed scope.
