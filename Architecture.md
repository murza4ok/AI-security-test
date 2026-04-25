# AI-security-test Architecture

## Purpose

The repository contains two separate runtime subsystems:
- `ai-sec` as the attacking CLI runtime
- `web_target` as the demo target web runtime

They live in one monorepo, but they do not form a single in-process runtime.
This document describes the runtime boundary first and then the internal
architecture of the current `ai-sec` runtime.

## Runtime Boundary Contract

Runtime ownership:
- `ai-sec` starts from `src/main.rs` via `cargo run --bin ai-sec -- ...`
- `web_target` starts from `src/bin/web_target.rs` via `cargo run --bin web_target --`

Supported interaction model:
- `ai-sec` talks either to model providers directly or to `web_target` through
  an external HTTP client
- `web_target` serves browser and HTTP traffic as an independent process
- `ai-sec -> web_target` integration goes only through external HTTP endpoints
  and request/response contracts

Allowed shared contract layer:
- HTTP surface of `web_target`: `/health`, `/login`, `/chat`, `/api/chat`,
  `/logout`
- synthetic fixtures and scenario manifests as shared data/schema inputs
- report metadata and evaluation schema where both runtimes need comparable
  evidence
- shared `.env` values for local demo convenience

Not part of the contract:
- direct calls from `ai-sec` into `src/bin/webapp/*`
- embedding `web_target` into the `ai-sec` process
- documentation that implies one monolithic runtime with two entry points

---

## High-Level Runtime View

```mermaid
flowchart TD
    U["Operator"] --> CLI["ai-sec binary"]
    B["Browser user"] --> WEB["web_target binary"]

    CLI --> P["LLM providers"]
    CLI --> API["web_target HTTP API"]
    WEB --> API
    WEB --> DATA["Synthetic app data / policy layer"]

    S["Shared contract layer"] --> API
    S --> FX["Fixtures / scenario manifests"]
    S --> R["Report metadata / schemas"]
```

---

## web_target Module Layout

`web_target` stays a separate runtime rooted at `src/bin/web_target.rs`, but its
internal structure is now split by responsibility:

- `src/bin/webapp/handlers.rs` — HTTP routes and the stable request/response
  contract for browser and API entry points
- `src/bin/webapp/auth.rs` — cookie parsing and session persistence
- `src/bin/webapp/state.rs` — fixture loading, synthetic support dataset, and
  transcript storage
- `src/bin/webapp/tools.rs` — tool-like backend operations such as customer
  lookup, ticket search, internal note access, and guarded redaction
- `src/bin/webapp/policy.rs` — intent classification and profile-aware allow/deny
  decisions
- `src/bin/webapp/html.rs` — server-rendered login/chat pages

This keeps the web target self-contained and suitable for HTTP-client
integration without exposing internal Rust modules across runtime boundaries.

## web_target API Contract

Current external surface:

- `GET /health` → `{"status":"ok","target":"acme-support-web"}`
- `GET /login` → login page for demo users and security profiles
- `POST /login` → sets session cookie and redirects to `/chat`
- `GET /chat` / `POST /chat` → browser flow backed by the same policy layer
- `POST /api/chat` → session-backed JSON chat endpoint used by `ai-sec`
  HTTP-target mode
- `POST /logout` → clears session cookie and redirects to `/login`

`POST /api/chat` request body:

```json
{"message":"show my ticket issue"}
```

`POST /api/chat` success response:

```json
{
  "user": "customer_alice",
  "profile": "naive",
  "answer": "…",
  "tool_calls_attempted": ["get_customer_summary", "search_tickets"],
  "tool_calls_allowed": ["get_customer_summary", "search_tickets"],
  "tool_calls_denied": [],
  "redactions": []
}
```

Error contract:

- no session → `401 {"error":"session-required"}`
- invalid request handled by policy → `400 {"error":"..."}`

The response field set above is treated as stable for the next HTTP integration
step; internal module changes should preserve it.

---

## ai-sec Module Layout

```mermaid
flowchart LR
    M["src/main.rs"] --> CLI["src/cli"]
    M --> CFG["src/config"]
    M --> APP["src/app"]

    APP --> APP1["interactive.rs"]
    APP --> APP2["runtime.rs"]
    APP --> APP3["providers.rs"]
    APP --> APP4["scenarios.rs"]
    APP --> APP5["target.rs"]

    APP2 --> ATK["src/attacks"]
    APP2 --> PAY["src/payloads"]
    APP2 --> GEN["src/generator"]
    APP2 --> SCN["src/scenarios"]
    APP2 --> ENG["src/engine"]
    APP2 --> PRV["src/providers"]
    APP2 --> RPT["src/reporting"]
    APP1 --> EDU["src/education"]

    ATK --> ATK1["registry.rs + attack families"]
    PAY --> PAY1["loader.rs"]
    GEN --> GEN1["generated payload mode"]
    SCN --> SCN1["builder / loader / retrieval / evaluator"]
    ENG --> ENG1["runner.rs / evaluator.rs / session.rs"]
    PRV --> PRV1["traits.rs + provider implementations"]
    RPT --> RPT1["terminal_report.rs / json_report.rs"]
```

---

## ai-sec Main Workflow

```mermaid
flowchart TD
    A["User runs ai-sec"] --> B["Cli::parse() in src/main.rs"]
    B --> C["AppConfig::from_env()"]
    C --> D["app::run(cli, app_config)"]

    D --> E1["interactive::run_interactive() when no subcommand"]
    D --> E2["runtime::run_command() when subcommand is present"]

    E2 --> F["Resolve provider(s) or HTTP target, attacks, optional scenario, optional generated mode"]
    F --> G["Load payloads / scenario assets"]
    G --> H["Create AttackRunner"]
    H --> I["Run attacks per provider"]
    I --> J["provider.complete(...)"]
    J --> K["engine evaluator + scenario evaluator"]
    K --> L["Build TestSession"]
    L --> M["terminal_report"]
    L --> N["json_report"]
```

---

## ai-sec Runtime Sequence

```mermaid
sequenceDiagram
    participant U as User
    participant CLI as main/CLI
    participant CFG as AppConfig
    participant REG as Attack Registry
    participant LDR as PayloadLoader
    participant RUN as AttackRunner
    participant PRV as Provider
    participant EV as Evaluator
    participant SES as TestSession
    participant RPT as Reporting

    U->>CLI: cargo run --bin ai-sec -- run --attack jailbreaking --provider openai
    CLI->>CFG: from_env()
    CLI->>REG: find_attack()
    CLI->>LDR: load_category()
    CLI->>RUN: run_session()
    RUN->>PRV: complete(system_prompt, prompt, request_config)
    PRV-->>RUN: LLMResponse / ProviderError
    RUN->>EV: evaluate(response, payload)
    EV-->>RUN: EvaluationResult
    RUN->>SES: add AttackResult / AttackRun
    SES-->>RPT: session data
    RPT-->>U: terminal table + JSON file
```

---

## Provider Layer

The provider layer abstracts differences between APIs.

Responsibilities:
- create HTTP clients with shared timeout settings
- apply retry/backoff policy
- map provider-specific HTTP responses into common `ProviderError`
- return a normalized `LLMResponse`

Model providers:
- OpenAI
- Anthropic
- DeepSeek
- YandexGPT
- Ollama

External target transport:
- `HttpTargetProvider` logs in to `web_target`, persists the session cookie,
  sends requests to `/api/chat`, and returns normalized `LLMResponse` values
  back into the existing attack engine
- the same provider-like abstraction also exposes session-level `target`
  metadata for JSON and terminal reports

Retry policy:
- retries only on timeout, transport/network failure, and HTTP `429`
- does not retry auth failures, parse failures, or non-retryable API errors

```mermaid
flowchart TD
    A["AttackRunner"] --> B["LLMProvider trait"]
    B --> C1["OpenAIProvider"]
    B --> C2["AnthropicProvider"]
    B --> C3["DeepSeekProvider"]
    B --> C4["YandexGptProvider"]
    B --> C5["OllamaProvider"]
    B --> C6["HttpTargetProvider"]

    C1 --> D["shared HTTP client"]
    C2 --> D
    C3 --> D
    C4 --> D
    C5 --> D
    C6 --> D

    D --> E["timeout"]
    D --> F["retry/backoff"]
```

---

## Payload System

Payloads are stored outside code in TOML files:
- one directory per attack family
- one or more TOML files per family
- each file contains metadata + payload entries

This makes the tool easy to extend without changing Rust code for every new case.

```mermaid
flowchart TD
    A["payloads/<attack>/*.toml"] --> B["PayloadLoader::load_category"]
    B --> C["read file contents"]
    C --> D["toml::from_str"]
    D --> E["PayloadFile"]
    E --> F["Vec<Payload>"]
    F --> G["Attack implementation"]
```

---

## Attack Execution Model

Each attack module implements the common `Attack` trait.

Responsibilities of an attack implementation:
- provide metadata: `id`, `name`, `description`
- load its payloads
- execute payloads against a provider
- emit `AttackResult`

The current design is mostly payload-driven:
- `PromptInjectionAttack` exposes shared execution logic reused by other attack families
- many attacks are still effectively single-request scenarios
- multi-turn realism is still limited and is an important future direction

```mermaid
flowchart TD
    A["Attack::execute"] --> B["iterate payloads"]
    B --> C["provider.complete(...)"]
    C --> D{"response?"}
    D -- ok --> E["evaluate(response, payload)"]
    D -- error --> F["Inconclusive / error text"]
    E --> G["AttackResult"]
    F --> G
    G --> H["on_result callback"]
    H --> I["collect into AttackRun"]
```

---

## Evaluation Logic

The evaluator is heuristic. It does not establish ground truth. It classifies
responses using:
- refusal signals
- success indicators
- response length / substantive content
- `harm_level`

`harm_level` controls interpretation:
- `L0` → informational, not a bypass
- `L1` → review-only, capped at partial
- `L2/L3` → scoreable safety failures

```mermaid
flowchart TD
    A["LLMResponse + Payload"] --> B["normalize text"]
    B --> C["match refusal signals"]
    B --> D["match success indicators"]
    C --> E{"refusal found?"}
    D --> F{"success found?"}

    E -- yes --> G{"mixed signals?"}
    G -- yes --> H["PARTIAL"]
    G -- no --> I["REFUSED"]

    E -- no --> J{"harm_level"}
    J -- L0 --> K["INFO"]
    J -- L1 --> L["PARTIAL review-only"]
    J -- L2/L3 --> M{"success + substantive?"}
    M -- yes --> N["BYPASS"]
    M -- no --> O["PARTIAL / INCONCLUSIVE"]
```

---

## Reporting Model

The reporting layer has two outputs:
- terminal summary/review
- JSON report for later comparison

The JSON schema now includes:
- `schema_version`
- provider metadata
- runtime configuration
- benchmark metadata
- per-attack derived metrics
- per-result metadata like `harm_level` and `model_used`

This is important for later diff/benchmark functionality.

```mermaid
flowchart TD
    A["TestSession"] --> B["schema_version"]
    A --> C["provider"]
    A --> D["config"]
    A --> E["benchmark"]
    A --> F["summary"]
    A --> G["attacks_run[]"]

    G --> H["AttackRun"]
    H --> H1["counters"]
    H --> H2["scoreable_payloads"]
    H --> H3["bypass_rate_pct"]
    H --> H4["results[]"]

    H4 --> I["AttackResult"]
    I --> I1["prompt_sent"]
    I --> I2["response_received"]
    I --> I3["harm_level"]
    I --> I4["evaluation"]
    I --> I5["model_used / tokens / latency"]
```

---

## Current Strengths

- clear modular layout
- provider abstraction is already in place
- payloads are externalized into TOML
- reports are persisted for later analysis
- benchmark-oriented metadata is now present in JSON
- retry/backoff is centralized instead of duplicated ad hoc

---

## Current Limitations

- evaluator is still heuristic and keyword-driven
- most attack execution is still effectively single-turn
- there is no dedicated `diff` command yet
- many-shot and context manipulation are not modeled as true session attacks
- no benchmark mode with fixed run profiles yet

---

## Recommended Next Steps

1. Add explicit `diff` between two JSON reports.
2. Add benchmark profiles: `quick`, `baseline`, `full`.
3. Move from single-prompt attacks to multi-turn/session scenarios.
4. Add payload validation and dataset hygiene checks.
5. Improve evaluator with review queues or judge-assisted classification.

---

## Key Files

- Entry point: [src/main.rs](src/main.rs)
- Config: [src/config/mod.rs](src/config/mod.rs)
- Provider trait: [src/providers/traits.rs](src/providers/traits.rs)
- Provider helpers: [src/providers/mod.rs](src/providers/mod.rs)
- Runner: [src/engine/runner.rs](src/engine/runner.rs)
- Evaluator: [src/engine/evaluator.rs](src/engine/evaluator.rs)
- Session/report model: [src/engine/session.rs](src/engine/session.rs)
- JSON report: [src/reporting/json_report.rs](src/reporting/json_report.rs)
- Terminal reporting: [src/reporting/terminal_report.rs](src/reporting/terminal_report.rs)
- Attack registry: [src/attacks/registry.rs](src/attacks/registry.rs)
- Payload loader: [src/payloads/loader.rs](src/payloads/loader.rs)
