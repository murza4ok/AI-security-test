# ai-sec Architecture

## Purpose

`ai-sec` is a Rust CLI tool for educational testing of LLM security behavior.
It loads attack payloads from TOML files, sends them to configured providers,
evaluates model responses heuristically, and stores terminal + JSON reports for
later comparison.

---

## High-Level Architecture

```mermaid
flowchart TD
    A["User / Operator"] --> B["CLI / Interactive Menu"]
    B --> C["main.rs"]

    C --> D["config"]
    C --> E["attacks registry"]
    C --> F["payload loader"]
    C --> G["providers"]
    C --> H["engine runner"]
    C --> I["reporting"]
    C --> J["education"]

    D --> D1[".env / environment"]
    E --> E1["attack implementations"]
    F --> F1["payloads/<attack>/*.toml"]
    G --> G1["OpenAI"]
    G --> G2["Anthropic"]
    G --> G3["DeepSeek"]
    G --> G4["YandexGPT"]
    G --> G5["Ollama"]
    H --> H1["evaluator"]
    H --> H2["session model"]
    I --> I1["terminal report"]
    I --> I2["JSON report"]
```

---

## Module Layout

```mermaid
flowchart LR
    M["main.rs"] --> CLI["src/cli"]
    M --> CFG["src/config"]
    M --> ATK["src/attacks"]
    M --> PAY["src/payloads"]
    M --> ENG["src/engine"]
    M --> PRV["src/providers"]
    M --> RPT["src/reporting"]
    M --> EDU["src/education"]

    ENG --> ENG1["runner.rs"]
    ENG --> ENG2["evaluator.rs"]
    ENG --> ENG3["session.rs"]

    PRV --> PRV1["traits.rs"]
    PRV --> PRV2["provider implementations"]

    PAY --> PAY1["loader.rs"]
    PAY --> PAY2["template.rs"]

    ATK --> ATK1["registry.rs"]
    ATK --> ATK2["attack files"]
```

---

## Main Workflow

```mermaid
flowchart TD
    A["User runs ai-sec"] --> B["CLI parsing"]
    B --> C["Load AppConfig from .env / env"]
    C --> D["Build provider instances"]
    D --> E["Resolve selected attack categories"]
    E --> F["Load payloads from TOML"]
    F --> G["Create AttackRunner"]
    G --> H["Run attacks per provider"]
    H --> I["Send payloads to model"]
    I --> J["Evaluate responses"]
    J --> K["Aggregate into TestSession"]
    K --> L["Print terminal summary"]
    K --> M["Write JSON report"]
```

---

## Runtime Sequence

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

    U->>CLI: ai-sec run --attack jailbreaking --provider openai
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

Current providers:
- OpenAI
- Anthropic
- DeepSeek
- YandexGPT
- Ollama

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

    C1 --> D["shared HTTP client"]
    C2 --> D
    C3 --> D
    C4 --> D
    C5 --> D

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

- Entry point: [src/main.rs](/E:/repos/AI-security-test/src/main.rs)
- Config: [src/config/mod.rs](/E:/repos/AI-security-test/src/config/mod.rs)
- Provider trait: [src/providers/traits.rs](/E:/repos/AI-security-test/src/providers/traits.rs)
- Provider helpers: [src/providers/mod.rs](/E:/repos/AI-security-test/src/providers/mod.rs)
- Runner: [src/engine/runner.rs](/E:/repos/AI-security-test/src/engine/runner.rs)
- Evaluator: [src/engine/evaluator.rs](/E:/repos/AI-security-test/src/engine/evaluator.rs)
- Session/report model: [src/engine/session.rs](/E:/repos/AI-security-test/src/engine/session.rs)
- JSON report: [src/reporting/json_report.rs](/E:/repos/AI-security-test/src/reporting/json_report.rs)
- Terminal reporting: [src/reporting/terminal_report.rs](/E:/repos/AI-security-test/src/reporting/terminal_report.rs)
- Attack registry: [src/attacks/registry.rs](/E:/repos/AI-security-test/src/attacks/registry.rs)
- Payload loader: [src/payloads/loader.rs](/E:/repos/AI-security-test/src/payloads/loader.rs)
