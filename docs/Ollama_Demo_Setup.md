# Ollama Demo Setup

## Purpose

This document describes the live local `Ollama` workflow for the current
repository state.

Use it when you want to run `ai-sec` directly against a local model without
any Windows-specific helpers, fixed absolute paths, or missing generator
scripts.

## Scope

The supported contract is the same as in the rest of the project:

- run from the repository root;
- launch `ai-sec` with `cargo run --bin ai-sec -- ...`;
- treat `web_target` as a separate process started only when HTTP mode is
  needed.

This guide does not assume any `scripts/` directory or repo-pinned model
storage path.

## Prerequisites

- `ollama` is installed and reachable;
- at least one local model is available, for example `qwen2.5:0.5b`;
- the current shell is in the repository root;
- `.env` is configured if you want a default provider/model without passing
  flags each time.

Quick checks:

```bash
ollama list
ollama ps
```

## Minimal Provider Validation

Before attacks, verify that the `ollama` provider is reachable by `ai-sec`:

```bash
cargo run --bin ai-sec -- check --provider ollama
```

Useful companion commands:

```bash
cargo run --bin ai-sec -- list
cargo run --bin ai-sec -- help run
```

## Minimal Direct Run

Single-category smoke against a local model:

```bash
cargo run --bin ai-sec -- run --attack jailbreaking --provider ollama --limit 1
```

If you prefer an explicit report path:

```bash
cargo run --bin ai-sec -- run --attack jailbreaking --provider ollama --limit 1 --output /tmp/ollama-direct.json
cargo run --bin ai-sec -- review /tmp/ollama-direct.json
```

## Scenario Demo

The current repository already contains live scenario fixtures. The shortest
practical scenario demo is:

```bash
cargo run --bin ai-sec -- run --attack sensitive_data_exposure --provider ollama --app-scenario support_bot --limit 1 --output /tmp/ollama-support-bot.json
cargo run --bin ai-sec -- review /tmp/ollama-support-bot.json
```

Other built-in scenarios:

- `hr_bot`
- `internal_rag_bot`
- `support_bot_hardened`

For `internal_rag_bot`, use explicit retrieval mode when needed:

```bash
cargo run --bin ai-sec -- run --attack sensitive_data_exposure --provider ollama --app-scenario internal_rag_bot --retrieval-mode subset --limit 1
```

## Multi-Turn Demo

The repo now has live multi-turn payload support. A minimal chain smoke is:

```bash
cargo run --bin ai-sec -- run --attack prompt_injection --provider ollama --limit 1 --output /tmp/ollama-multiturn.json
cargo run --bin ai-sec -- review /tmp/ollama-multiturn.json
```

## Optional Custom Local Model

The repository includes [`AcmeSMBSupport7B.Modelfile`](../AcmeSMBSupport7B.Modelfile).
If you want to build a custom local model persona, create it explicitly and
then use `--model` together with `--provider ollama`.

Example shape:

```bash
ollama create acme-smb-support-7b -f AcmeSMBSupport7B.Modelfile
cargo run --bin ai-sec -- run --attack sensitive_data_exposure --provider ollama --model acme-smb-support-7b --app-scenario support_bot --limit 1
```

Only keep this workflow if the model build actually succeeds in your
environment.

## Review And Compare

Saved reports can be compared directly:

```bash
cargo run --bin ai-sec -- compare /tmp/ollama-direct.json /tmp/ollama-support-bot.json /tmp/ollama-multiturn.json
```

`review` shows the detailed payload-level evidence:

- scenario exposure metadata;
- HTTP target metadata when present;
- tool decisions and redactions;
- multi-turn transcript details.

`compare` is the higher-level session summary view for:

- provider/model/mode comparison;
- target or scenario labels;
- exposure, chain, and request counts;
- per-attack bypass percentages across saved reports.

## What Is Intentionally Not Documented Here

The following older assumptions are no longer part of the supported workflow:

- fixed absolute repo paths;
- PowerShell-only setup steps;
- generated helper scripts in `buffer_for_ollama/`;
- a mandatory repo-local `OLLAMA_MODELS` directory;
- instructions that describe `sensitive_data_exposure` as a future feature.
