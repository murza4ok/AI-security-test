# Bundle Start Here

## Что это

Этот репозиторий подготовлен как приватный transfer-snapshot для продолжения работы в другой Codex-сессии через `git bundle`.

Это не публичный релиз и не чистый merge-ready mainline.

Это рабочий WIP-снимок, в котором важно сохранить:

- текущий локальный код и fixtures;
- документацию и технический аудит;
- roadmap и branch orchestration;
- task-pack для поэтапного запуска writer-agent по веткам.

## Почему bundle собран как полный snapshot

Для продолжения работы недостаточно перенести только сегодняшние документы.

В локальном состоянии репозитория уже есть не только planning-слой, но и актуальные рабочие файлы, которых может не быть в публичном remote:

- `docs/*`
- `src/bin/*`
- `vendor/*`
- текущие fixtures и локальные изменения runtime/scenario/reporting слоёв

Если перенести только документацию, новая Codex-сессия увидит планы, но не увидит реальное состояние проекта, на которое эти планы опираются.

Поэтому bundle нужно воспринимать как полный приватный snapshot текущего рабочего дерева, а не только как перенос “сегодняшних заметок”.

## Что было сделано сегодня

Сегодня в репозитории был добавлен организационный слой для weekend-итерации:

- `refactoring.md` — технический аудит проекта;
- `TZ.md` — целевое техническое задание;
- `Roadmap_weekend.md` — верхнеуровневый weekend roadmap;
- `Branch_tasks.md` — разбиение roadmap на ветки разработки;
- `development/*` — task-pack и иерархия запуска writer-agent;
- cleanup и актуализация документации;
- фиксация того, что `ai-sec` и `web_target` должны рассматриваться как отдельные runtime-контура.

## С чего начать новой Codex-сессии

Прочитать документы в таком порядке:

1. `BUNDLE_START_HERE.md`
2. `development/README.md`
3. `refactoring.md`
4. `TZ.md`
5. `Roadmap_weekend.md`
6. `Branch_tasks.md`
7. `development/branches/00-weekend-integration/task.md`
8. `development/branches/01-runtime-boundary-contract/task.md`

После этого уже смотреть:

- `README.md`
- `Architecture.md`
- `docs/README.md`
- релевантные `task.md` для следующих веток

## Текущий статус weekend-плана

Step 0 завершён:

- собран orchestration-layer;
- созданы task-pack для всех веток;
- подготовлен process-contract для writer-agent и reviewer-agent.

Фактическая feature-разработка по weekend-веткам ещё не начата.

Следующий правильный шаг:

1. создать или продолжить работу от `codex/weekend-integration`;
2. начать ветку `codex/runtime-boundary-contract`;
3. работать по `development/branches/01-runtime-boundary-contract/task.md`.

## Как интерпретировать структуру `development/`

- `development/README.md` — главный orchestration-файл;
- `development/branches/00-*` — управляющий task-pack интеграционной ветки;
- `development/branches/01-*` … `11-*` — отдельные task-pack под feature-ветки.

Каждый `task.md` уже содержит:

- цель;
- зависимости;
- allowed scope;
- forbidden scope;
- required outcomes;
- mandatory checks;
- handoff;
- stop/escalate conditions.

## Что не надо делать новой сессии

- не запускать несколько writer-agent в одну ветку;
- не начинать с произвольной feature-ветки мимо `01-runtime-boundary-contract`;
- не вести работу напрямую в `main`;
- не пытаться “сразу делать весь roadmap” без соблюдения wave-order.

## Ожидаемый режим продолжения

Нормальная схема продолжения такая:

1. импортировать bundle;
2. открыть этот файл;
3. пройти read-order выше;
4. создать `codex/weekend-integration`;
5. запускать writer-agent по одному `task.md` за раз;
6. после каждой ветки требовать handoff и только потом делать merge.
