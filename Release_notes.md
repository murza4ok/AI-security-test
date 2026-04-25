# Release Notes

## Weekend Sprint Release

Итоговая ветка: `codex/weekend-integration`

Статус:

- weekend-план `01`–`11` завершён;
- финальный integration smoke пройден;
- документация и status-tracking синхронизированы.

## Main Changes

### Runtime Contract

- `ai-sec` и `web_target` зафиксированы как отдельные runtime-подсистемы.
- Поддерживаемый запуск выровнен на `cargo run --bin ai-sec -- ...` и `cargo run --bin web_target --`.
- Убраны ложные монолитные допущения в коде и документации.

### CLI And DX

- Help-path и CLI examples приведены к фактическому контракту.
- Интерактивный режим больше не падает целиком без настроенного provider.
- Честно задокументированы `--output`, `--model`, `--app-scenario` и HTTP target flags.

### Runtime And Providers

- Исправлены deterministic payload loading и `--limit`.
- Нормализован report contract для generated payloads.
- Provider layer выпрямлен через общий factory path и shared OpenAI-compatible слой.
- Override `--model` теперь работает предсказуемо и безопасно.

### Scenario Mode

- Зафиксирован честный scenario contract.
- Добавлены `real_envelopes` и `meta_envelopes` в report path.
- `session_seed`, retrieval subset и PII masking приведены к рабочей семантике.

### Web Target And HTTP Attacks

- `web_target` разделён на handlers/auth/state/policy/tools/html layers.
- `ai-sec` получил HTTP target mode через `/login` + `/api/chat`.
- В report path сохраняются target metadata, tool calls, redactions и request counts.

### Multi-Turn

- Добавлен multi-turn payload format с цепочками `2..5` шагов.
- Для результатов сохраняются transcript, chain counters и abort reasons.
- Multi-turn path работает как для direct provider runs, так и для HTTP target runs.

### Reporting

- Summary и derived metrics переведены на единый source of truth.
- Review и compare расширены под scenario, HTTP target и multi-turn runs.
- JSON migration path выровнен для legacy report loading.

### Documentation

- Проведён docs consistency sweep.
- Удалены устаревшие и неподдерживаемые документы.
- Добавлены итоговые документы спринта и прозрачный status tracker между сессиями.

## Verification

На итоговой интеграционной базе подтверждены:

- `cargo check --offline --all-targets`
- `cargo test --offline`
- `cargo run --offline --bin ai-sec -- list`
- direct provider run
- scenario-driven run
- HTTP attack run
- `review`
- `compare`
- `curl -i http://127.0.0.1:3000/health`

## Known Limitations

- adaptive generation по промежуточным ответам пока не доведена до полного production-ready контура;
- `web_target` использует фиксированный локальный порт `3000`;
- отдельный `--help` path у `web_target` отсутствует;
- финальный article-ready benchmark matrix ещё не оформлен отдельным reproducible пакетом.

## Recommended Next Iteration

1. Довести adaptive attack loop.
2. Собрать формальный demo-matrix для статьи.
3. Усилить route-level и integration tests.
4. Убрать жёсткую привязку `web_target` к фиксированному порту.
