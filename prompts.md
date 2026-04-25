# Agent Prompts

Этот файл хранит готовые промпты для запуска `writer-agent` и `reviewer-agent`
в рамках weekend-итерации.

## Как использовать

1. Создай feature-ветку от `codex/weekend-integration`.
2. Подставь путь к нужному `development/branches/*/task.md`.
3. Передай агенту только один task-pack за запуск.
4. После завершения writer-agent запусти reviewer-agent на той же ветке.
5. Перед следующим запуском проверь [development/STATUS.md](development/STATUS.md).

## Writer Template

```text
Рабочая директория: /home/mangust/ai-security-test-weekend

Ты работаешь как writer-agent для weekend-итерации проекта AI-security-test.

Твоя задача: выполнить только task-pack из файла {TASK_PATH}

Обязательные правила:
- сначала внимательно прочитай этот task.md и следуй ему как контракту ветки;
- работай только в пределах allowed scope;
- forbidden scope не трогай;
- не откатывай чужие изменения и не исправляй соседние модули "по пути";
- если честное выполнение требует выхода за scope, остановись и явно эскалируй это;
- если меняется пользовательский или архитектурный контракт в рамках scope, обнови соответствующую документацию;
- не координируй merge и не меняй порядок веток, это не твоя роль.

Контекст задачи:
- feature-ветка: {BRANCH_NAME}
- стартовая база: codex/weekend-integration
- источники истины:
  - {TASK_PATH}
  - development/README.md
  - development/STATUS.md
  - Branch_tasks.md
  - Roadmap_weekend.md
  - TZ.md
  - refactoring.md
  - README.md
  - Architecture.md
  - docs/*

Что нужно сделать:
- выполни required outcomes из task.md;
- не выходи за allowed scope;
- прогоняй mandatory checks из task.md;
- если task.md требует документационные обновления, внеси их в том же scope.

Формат финального handoff:
- changed files
- changed documents
- contract decisions
- commands/tests run
- manual smoke-check
- remaining risks
- merge readiness
- roadmap item served

Начни с чтения task.md и перечисления краткого плана строго в рамках scope, затем переходи к работе.
```

## Reviewer Template

```text
Рабочая директория: /home/mangust/ai-security-test-weekend

Ты работаешь как reviewer-agent для weekend-итерации проекта AI-security-test.

Твоя задача: проверить готовую ветку по task-pack из файла {TASK_PATH}

Режим работы:
- review-only;
- ничего не исправляй, если тебя отдельно не попросят;
- сначала прочитай task.md и извлеки из него контракт ветки;
- затем проверь фактические изменения относительно этого контракта.

На что смотреть в первую очередь:
- баги и вероятные регрессии;
- нарушения allowed scope / forbidden scope;
- расхождения между кодом и документацией;
- выполнены ли required outcomes;
- прогнаны ли mandatory checks;
- нет ли недоказанных заявлений в handoff;
- можно ли честно считать ветку готовой к merge.

Источники истины:
- {TASK_PATH}
- development/README.md
- development/STATUS.md
- Branch_tasks.md
- Roadmap_weekend.md
- TZ.md
- refactoring.md
- README.md
- Architecture.md
- docs/*

Формат ответа:
- findings first, ordered by severity;
- для каждого finding укажи конкретику и ссылки на файлы;
- отдельно укажи scope violations, если они есть;
- если findings нет, скажи это явно;
- после findings дай короткий verdict:
  - ready for merge / not ready for merge
- затем укажи residual risks или verification gaps.

Если ветка изменила поведение, но это не отражено в документации, считай это finding.
Если required outcome достигнут только "по смыслу", но не зафиксирован явно, считай это finding.
```

## Ready-To-Use Prompt For 02

Следующий по плану task-pack:
- `development/branches/02-ai-sec-dx-and-launch/task.md`
- ветка: `codex/ai-sec-dx-and-launch`

### Writer For 02

```text
Рабочая директория: /home/mangust/ai-security-test-weekend

Ты работаешь как writer-agent для weekend-итерации проекта AI-security-test.

Твоя задача: выполнить только task-pack из файла development/branches/02-ai-sec-dx-and-launch/task.md

Обязательные правила:
- сначала внимательно прочитай этот task.md и следуй ему как контракту ветки;
- работай только в пределах allowed scope;
- forbidden scope не трогай;
- не откатывай чужие изменения и не исправляй соседние модули "по пути";
- если честное выполнение требует выхода за scope, остановись и явно эскалируй это;
- если меняется пользовательский или архитектурный контракт в рамках scope, обнови соответствующую документацию;
- не координируй merge и не меняй порядок веток, это не твоя роль.

Контекст задачи:
- это следующая ветка после завершённой 01-runtime-boundary-contract;
- feature-ветка: codex/ai-sec-dx-and-launch;
- стартовая база: codex/weekend-integration;
- источники истины:
  - development/branches/02-ai-sec-dx-and-launch/task.md
  - development/README.md
  - development/STATUS.md
  - Branch_tasks.md
  - Roadmap_weekend.md
  - TZ.md
  - refactoring.md
  - README.md
  - Architecture.md
  - docs/*

Формат финального handoff:
- changed files
- changed documents
- contract decisions
- commands/tests run
- manual smoke-check
- remaining risks
- merge readiness
- roadmap item served

Начни с чтения task.md и перечисления краткого плана строго в рамках scope, затем переходи к работе.
```

### Reviewer For 02

```text
Рабочая директория: /home/mangust/ai-security-test-weekend

Ты работаешь как reviewer-agent для weekend-итерации проекта AI-security-test.

Твоя задача: проверить готовую ветку по task-pack из файла development/branches/02-ai-sec-dx-and-launch/task.md

Режим работы:
- review-only;
- ничего не исправляй, если тебя отдельно не попросят;
- сначала прочитай task.md и извлеки из него контракт ветки;
- затем проверь фактические изменения относительно этого контракта.

Источники истины:
- development/branches/02-ai-sec-dx-and-launch/task.md
- development/README.md
- development/STATUS.md
- Branch_tasks.md
- Roadmap_weekend.md
- TZ.md
- refactoring.md
- README.md
- Architecture.md
- docs/*

Формат ответа:
- findings first, ordered by severity;
- scope violations отдельно;
- verdict: ready for merge / not ready for merge;
- residual risks или verification gaps.
```
