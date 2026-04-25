# HTTP Target Mode

## Назначение

Этот режим позволяет атаковать `web_target` как внешнюю HTTP-цель, а не как внутренний Rust-модуль.

`ai-sec` общается только через внешний контракт:

- `POST /login`
- `POST /api/chat`

## CLI Contract

Базовый запуск:

```bash
cargo run --bin ai-sec -- run \
  --attack prompt_injection \
  --target-mode http \
  --target-base-url http://127.0.0.1:3000 \
  --target-user customer_alice \
  --target-profile naive
```

Обязательные флаги HTTP mode:

- `--target-mode http`
- `--target-base-url`
- `--target-user`
- `--target-profile`

Поведение:

- `ai-sec` делает login через `POST /login`;
- session cookie сохраняется внутри одного run и переиспользуется для следующих запросов к `/api/chat`;
- при `401 session-required` клиент сбрасывает cookie, логинится заново и повторяет запрос один раз;
- JSON report сохраняет `target` metadata: mode, base URL, endpoint, authenticated user, profile, session persistence, request count, tool calls, redactions.

## Совместимость

На текущем этапе HTTP mode предназначен для classic payload-driven атак:

- `prompt_injection`
- `jailbreaking`
- `extraction`
- `goal_hijacking`
- `token_attacks`
- `many_shot`
- `context_manipulation`

Generated mode можно комбинировать с HTTP mode, если настроен generator provider для мутаций payload-ов.

Multi-turn chains тоже поддержаны, если payload описан через `[[payloads.turns]]` и содержит от `2` до `5` шагов. В HTTP mode `ai-sec` отправляет только текущий user turn, а история разговора живёт на стороне `web_target` через session cookie.

## Ограничения

- `sensitive_data_exposure` не поддерживается в HTTP mode;
- scenario-specific флаги (`--app-scenario`, `--fixture-root`, `--retrieval-mode`, `--scenario-config`, `--tenant`, `--session-seed`) в HTTP mode запрещены;
- `--provider` и `--model` в HTTP mode не используются;
- tenant не извлекается из текущего внешнего API `web_target`, поэтому в report остаётся `null`, если цель явно его не возвращает;
- transcript атаки и transcript самого web-target — это разные уровни состояния; текущий report хранит transcript со стороны `ai-sec`, а не HTML transcript страницы `/chat`.
