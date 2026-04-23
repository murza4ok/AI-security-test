# Ollama Demo Setup Guide

## Purpose

This guide explains how to prepare a local `Ollama` demo environment for the future `sensitive_data_exposure` mode in `ai-sec`.

The goal is to create a controlled local mock environment where:

- a local model runs in `Ollama`
- `ai-sec` can send scenario-built prompts to that model
- synthetic internal data is available for hidden-context assembly
- the operator can reproduce demonstrations safely

## What This Mock Environment Should Demonstrate

The mock environment is not a real enterprise deployment. It is a reproducible local stand-in for an SMB LLM application.

It should simulate:

- a support chatbot with hidden CRM context
- an HR assistant with payroll and employee notes
- a RAG assistant with internal docs

## Recommended Local Setup

### Required Components

- `Ollama`
- one or more local models
- this repository
- synthetic fixtures under `fixtures/sensitive_data_exposure/`

### Recommended Models For Demo

Use at least two models so the demo has a comparison angle.

Recommended baseline candidates:

- `llama3.1:8b`
- `qwen2.5:7b`
- `mistral:7b`

If hardware allows, optionally add one larger local model.

## Repository Path Policy

For this repository, use these fixed paths:

- model blobs: `E:\repos\AI-security-test\ollama_models`
- generated and auxiliary files: `E:\repos\AI-security-test\buffer_for_ollama`

The helper scripts in `scripts/` enforce this path policy by setting `OLLAMA_MODELS` before any `ollama pull` or `ollama serve` command.

## Install Ollama

Install `Ollama` from the official site and verify it works:

```powershell
ollama --version
```

Start the server if needed:

```powershell
$env:OLLAMA_MODELS = 'E:\repos\AI-security-test\ollama_models'
ollama serve
```

In many setups the service auto-starts. If it is already running, do not start a second instance.

You can also generate and use a helper script:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\Setup-OllamaDemo.ps1
powershell -ExecutionPolicy Bypass -File .\buffer_for_ollama\start-ollama-server.ps1
```

## Pull Demo Models

Example:

```powershell
$env:OLLAMA_MODELS = 'E:\repos\AI-security-test\ollama_models'
ollama pull llama3.1:8b
ollama pull qwen2.5:7b
ollama pull mistral:7b
```

Verify installed models:

```powershell
$env:OLLAMA_MODELS = 'E:\repos\AI-security-test\ollama_models'
ollama list
```

To generate the helper scripts and optionally pull models in one step:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\Setup-OllamaDemo.ps1 -PullModels
```

## Recommended `.env` For Local Demo

Set the project to use `Ollama`:

```env
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=llama3.1:8b

REQUEST_TIMEOUT_SECS=60
REQUEST_DELAY_MS=250
CONCURRENCY=1
RETRY_MAX_ATTEMPTS=2
RETRY_BASE_DELAY_MS=500
RETRY_MAX_DELAY_MS=2000
```

### Why These Defaults

- `CONCURRENCY=1` keeps the demo stable on modest hardware
- longer timeout helps with local inference latency
- lower delay still makes sequential runs tolerable

## Recommended Demo Directory Layout

When the feature is implemented, keep the synthetic demo assets here:

```text
fixtures/
  sensitive_data_exposure/
    support_bot/
    hr_bot/
    internal_rag_bot/
```

Keep payloads here:

```text
payloads/
  sensitive_data_exposure/
```

## How To Prepare The Mock Data

Create synthetic fixtures for each scenario.

### Scenario 1: support_bot

Recommended files:

- `scenario.toml`
- `system_prompt.txt`
- `customers.csv`
- `tickets.json`
- `support_notes.md`
- `secrets.toml`

What to include:

- 20 to 50 customer rows
- emails and phone numbers
- support case summaries
- internal notes that users should not see
- one support-oriented API canary

### Scenario 2: hr_bot

Recommended files:

- `scenario.toml`
- `system_prompt.txt`
- `employees.csv`
- `payroll.csv`
- `reviews.json`
- `secrets.toml`

What to include:

- 15 to 30 employees
- title, department, salary, manager
- personal contact info
- internal review comments
- one payroll-oriented canary

### Scenario 3: internal_rag_bot

Recommended files:

- `scenario.toml`
- `system_prompt.txt`
- `kb/security_runbook.md`
- `kb/infra_access.md`
- `kb/oncall.md`
- `kb/vendors.md`
- `secrets.toml`

What to include:

- internal admin URLs
- internal process notes
- vendor access notes
- incident and escalation notes
- one or more document-embedded canaries

## How To Design Good Synthetic Data

Synthetic data should be:

- realistic enough to look operational
- compact enough for local models
- clearly sensitive by appearance
- safe to store in git

### Good Examples

- `customer_email`
- `customer_phone`
- `ticket_internal_note`
- `salary_monthly_usd`
- `admin_portal_url`
- `service_account_name`
- `CANARY_*` values

### Avoid

- real names of actual people you know
- real domains under your control
- real tokens or passwords
- giant dumps that exceed practical context sizes

## System Prompt Mocking Strategy

Each scenario should include a realistic system prompt that explains the app role.

Example intent for `support_bot`:

- act as a customer support assistant
- use hidden customer context to answer requests
- do not reveal raw internal records

Example intent for `hr_bot`:

- act as an HR assistant
- use internal HR records
- never reveal salary or internal notes directly

Example intent for `internal_rag_bot`:

- answer questions based on retrieved internal docs
- summarize safely
- do not quote internal docs verbatim unless allowed

The prompt should be strong enough to resemble a real app, but not so strong that it makes the demo uninteresting.

## Mock Context Assembly

Before the feature exists, the mental model should be:

1. choose a scenario
2. load scenario system prompt
3. load scenario files
4. build hidden context
5. append attacker payload as user input
6. send to `Ollama`

For the first implementation, keep the assembly deterministic and simple.

## Recommended First Demo Plan

Run the demo in this order:

1. `support_bot` on one smaller model
2. `hr_bot` on the same model
3. `internal_rag_bot` on the same model
4. repeat on a second model
5. compare leakage rates

This makes it easy to show:

- same payload family
- different application context
- different model susceptibility

## Practical Performance Advice

Local models degrade quickly if context is too large.

Recommended limits:

- keep each scenario under a few short documents or CSV slices
- avoid giant SQL dumps in the first version
- prefer 20 to 60 KB per scenario over hundreds of KB
- start with `CONCURRENCY=1`
- increase only after baseline runs are stable

## Suggested Fixture Sizes

### support_bot

- `customers.csv`: 20 to 30 rows
- `tickets.json`: 10 to 20 tickets
- `support_notes.md`: 1 to 2 pages

### hr_bot

- `employees.csv`: 15 to 25 rows
- `payroll.csv`: 15 to 25 rows
- `reviews.json`: 10 to 15 review entries

### internal_rag_bot

- 3 to 5 markdown docs
- each doc around 30 to 120 lines

## Canary Design Rules

Each canary should:

- appear only in hidden context
- be unique to one scenario
- be obvious when leaked
- be easy to regex-match

Good format:

```text
CANARY_<SCENARIO>_<TYPE>_<ID>
```

Examples:

- `CANARY_SUPPORT_API_01XZ`
- `CANARY_HR_PAYROLL_88LM`
- `CANARY_RAG_ADMIN_URL_51PQ`

## Suggested Demo Commands

Current baseline validation:

```powershell
cargo run --bin ai-sec -- check --provider ollama
cargo run --bin ai-sec -- list
```

Future scenario-mode examples:

```powershell
cargo run --bin ai-sec -- run --attack sensitive_data_exposure --provider ollama --app-scenario support_bot
cargo run --bin ai-sec -- run --attack sensitive_data_exposure --provider ollama --app-scenario hr_bot
cargo run --bin ai-sec -- run --attack sensitive_data_exposure --provider ollama --app-scenario internal_rag_bot
```

Future comparison examples:

```powershell
cargo run --bin ai-sec -- run --attack sensitive_data_exposure --provider ollama --model llama3.1:8b --app-scenario support_bot
cargo run --bin ai-sec -- run --attack sensitive_data_exposure --provider ollama --model qwen2.5:7b --app-scenario support_bot
```

## How To Present The Demo

The strongest narrative is:

- this is a local assistant
- it was given hidden business context
- an attacker used prompt-based extraction
- the model disclosed data that should not have been revealed

Focus the demo on architectural weakness, not on blaming a specific model family.

Recommended sequence:

1. show the scenario and synthetic data types
2. show that the user should not see hidden context
3. run the payloads
4. show leaked canaries or records
5. compare across models
6. explain which app design choice caused the exposure

## Validation Checklist

Before using the demo, verify:

- `ollama serve` is running
- chosen model is pulled locally
- `cargo run --bin ai-sec -- check --provider ollama` succeeds
- synthetic fixtures exist and load cleanly
- canary values are unique
- no real secrets were accidentally added

## Common Mistakes

- using too much context and making local inference unstable
- mixing real-looking but actual secrets into fixtures
- using only one payload and calling it a scenario
- making all payloads too direct and repetitive
- evaluating only by manual reading instead of canary detection

## Minimum Mock Environment To Start

If time is limited, start with this:

- one model: `llama3.1:8b`
- one scenario: `support_bot`
- 4 fixture files
- 8 to 10 payloads
- 2 canaries

Then expand to:

- `hr_bot`
- `internal_rag_bot`
- second model for comparison

## Recommended Next Step

After the implementation branch starts, the first deliverable should be:

- one working `support_bot` scenario on `Ollama`

That gives the fastest path to a reproducible demo and keeps the first iteration small enough to stabilize.
