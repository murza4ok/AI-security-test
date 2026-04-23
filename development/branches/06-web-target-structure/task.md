# task.md

## Branch

`codex/web-target-structure`

## Wave

`Wave 2`, можно запускать параллельно с `03-ai-sec-runtime-determinism` и `04-provider-contract-refactor`.

## Mission

Довести `web_target` до модульной, самостоятельной и пригодной к развитию структуры.

## Dependencies

- базироваться на `codex/weekend-integration` после merge ветки `02-ai-sec-dx-and-launch`.

## Sources Of Truth

- `Branch_tasks.md`
- `Roadmap_weekend.md`
- `TZ.md`
- `Architecture.md`
- `README.md`

## Allowed Scope

- `src/bin/web_target.rs`
- `src/bin/webapp/*`
- `fixtures/sensitive_data_exposure/support_bot*`
- минимальные docs по web-target

## Forbidden Scope

- `src/scenarios/*`
- `src/providers/*`
- `src/engine/*`
- `src/reporting/*`

## Required Outcomes

- внутри `web_target` явно разделены auth/session, state/data access, policy, tool-like behavior и rendering;
- длинная policy-логика декомпозирована;
- API `/api/chat` стабилен;
- `web_target` сохраняет самостоятельность как отдельное web-приложение.

## Mandatory Checks

- `cargo check --all-targets`
- ручной запуск `web_target`
- smoke-check `/health`
- smoke-check `/login`
- smoke-check `/chat`
- smoke-check `/api/chat`

## Handoff

- перечислить изменённые web-target модули;
- описать новую внутреннюю структуру;
- указать финальный API contract `/api/chat`;
- перечислить ручные smoke-check;
- указать остаточные архитектурные риски.

## Stop And Escalate If

- для завершения задачи нужно менять `ai-sec` runtime или providers;
- стабилизация API требует отдельного контракта вне разрешённого scope.
