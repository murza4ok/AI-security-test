# task.md

## Branch

`codex/http-target-client`

## Wave

`Wave 4`, строго после `05-scenario-contract` и `06-web-target-structure`.

## Mission

Научить `ai-sec` атаковать `web_target` как внешнюю HTTP-цель, а не как внутренний Rust-модуль.

## Dependencies

- базироваться на `codex/weekend-integration` после merge веток `05-scenario-contract`, `06-web-target-structure` и `04-provider-contract-refactor`.

## Sources Of Truth

- `Branch_tasks.md`
- `Roadmap_weekend.md`
- `TZ.md`
- `README.md`
- `Architecture.md`
- документация web-target, если она появится в пределах этой ветки

## Allowed Scope

- новый target client или новый модуль target integration
- `src/app/*` в части нового режима запуска
- `src/providers/*` только если HTTP target интегрируется через provider-подобный слой
- `src/reporting/*` только в части target metadata
- docs по HTTP mode

## Forbidden Scope

- внутренняя policy-логика `web_target`, кроме необходимой фиксации API response contract
- `src/scenarios/*`, если это не связано напрямую с HTTP target metadata

## Required Outcomes

- реализован HTTP-клиент для `/api/chat`;
- поддержаны login flow и cookie/session persistence;
- добавлены CLI-параметры для base URL, user и profile;
- target metadata сохраняется в отчёт;
- `ai-sec` не зависит от внутренних модулей `web_target`.

## Mandatory Checks

- `cargo check --all-targets`
- `cargo test`
- ручной end-to-end smoke: поднять `web_target`
- ручной end-to-end smoke: выполнить атаку из `ai-sec` через HTTP
- ручной end-to-end smoke: проверить JSON report

## Handoff

- перечислить новые и изменённые target/client файлы;
- описать новый CLI contract HTTP mode;
- описать session persistence;
- перечислить проверки;
- указать ограничения текущей HTTP-интеграции.

## Stop And Escalate If

- для завершения задачи требуется глубокая переделка `web_target` policy layer;
- HTTP mode требует изменения scenario contract вне разрешённого scope.
