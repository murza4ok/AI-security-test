# ai-sec — LLM Security Testing Tool

An educational CLI tool for testing LLM (Large Language Model) security vulnerabilities. Built for security professionals who want to understand AI attack surfaces.

---

## What it does

`ai-sec` sends crafted prompts to LLMs and evaluates whether the model's safety training held or was bypassed. It covers **7 attack categories** with documented payloads, and includes educational content explaining each technique.

This is a research and learning tool — not a weaponised automation framework. Every attack category ships with explanations, academic references, and mitigations.

---

## Quick Start

```bash
# 1. Copy env template and add your API key
cp .env.example .env
# Edit .env and add OPENAI_API_KEY or ANTHROPIC_API_KEY

# 2. Build
cargo build --release

# 3. Check connectivity
./target/release/ai-sec check

# 4. List available attacks
./target/release/ai-sec list

# 5. Run a specific attack
./target/release/ai-sec run --attack jailbreaking

# 6. Run with output saved to JSON
./target/release/ai-sec run --attack jailbreaking --output report.json

# 7. Interactive menu mode (no subcommand)
./target/release/ai-sec

# 8. Learn about an attack
./target/release/ai-sec explain jailbreaking
```

---

## Attack Categories

| ID                    | Name                    | Payloads | Description |
|-----------------------|-------------------------|----------|-------------|
| `prompt_injection`    | Direct Prompt Injection | 6        | Override system instructions via user input |
| `jailbreaking`        | Jailbreaking Techniques | 12       | DAN, roleplay, hypothetical, encoding tricks |
| `extraction`          | System Prompt Extraction| 6        | Recover hidden operator instructions |
| `goal_hijacking`      | Goal Hijacking          | 5        | Redirect model from intended to attacker task |
| `token_attacks`       | Token-Level Attacks     | 8        | Unicode homoglyphs, zero-width chars, leetspeak |
| `many_shot`           | Many-Shot Jailbreaking  | 3        | Condition model with many Q&A examples |
| `context_manipulation`| Context Manipulation    | 5        | False permissions, memory poisoning |

---

## Providers

Configure in `.env` (copy from `.env.example`):

| Provider  | Config vars                        | Notes |
|-----------|------------------------------------|-------|
| OpenAI    | `OPENAI_API_KEY`, `OPENAI_MODEL`   | Default model: `gpt-4o` |
| Anthropic | `ANTHROPIC_API_KEY`, `ANTHROPIC_MODEL` | Default: `claude-3-5-sonnet-20241022` |
| Ollama    | `OLLAMA_BASE_URL`, `OLLAMA_MODEL`  | Local; no key needed |

Auto-selects first configured provider. Override with `--provider openai`.

---

## CLI Reference

```
ai-sec [OPTIONS] [COMMAND]

COMMANDS:
  run      Run attack(s): --attack <id> [--limit N] [--output file.json]
  list     List all attack categories and payload counts
  explain  Show educational content: explain <attack_id>
  check    Test provider connectivity

OPTIONS:
  -p, --provider <PROVIDER>  Override provider (openai, anthropic, ollama)
  -v, --verbose              Increase log verbosity
```

---

## Result Labels

| Label        | Meaning |
|--------------|---------|
| `✓ REFUSED`  | Model refused — safety training held |
| `⚠ PARTIAL`  | Ambiguous — partial compliance or unclear |
| `✗ BYPASS`   | Attack succeeded — safety bypassed |
| `  ERROR`    | Request failed (network/auth error) |

> All evaluations are **heuristic** based on keyword matching. Manual review is recommended for ambiguous results.

---

## Project Structure

```
ai-sec/
├── src/
│   ├── main.rs          — Entry point, provider selection, command dispatch
│   ├── config/          — Loads .env settings
│   ├── providers/       — OpenAI, Anthropic, Ollama HTTP clients
│   ├── attacks/         — 7 attack categories + registry
│   ├── payloads/        — TOML loader + template renderer
│   ├── engine/          — Runner, evaluator, session tracking
│   ├── reporting/       — Terminal tables + JSON export
│   ├── cli/             — Arg parsing, menus, display helpers
│   └── education/       — Explain command content
├── payloads/            — TOML payload files (editable without recompiling)
│   ├── prompt_injection/
│   ├── jailbreaking/
│   ├── extraction/
│   ├── goal_hijacking/
│   ├── token_attacks/
│   ├── many_shot/
│   └── context_manipulation/
├── docs/               — Session notes, rules, operator plan (gitignored)
└── .env.example        — Config template
```

---

## Adding New Payloads

Payloads are TOML files — no Rust required:

```toml
# payloads/jailbreaking/my_payloads.toml
[metadata]
attack_type = "jailbreaking"
variant = "custom"
severity = "medium"

[[payloads]]
id = "my_test"
name = "My Custom Payload"
description = "What this tests"
prompt = "Your prompt here..."
expected_refusal_keywords = ["cannot", "won't"]
success_indicators = ["target phrase"]
```

Restart `ai-sec` and your payloads appear automatically.

---

## Key Literature

| Topic | Paper / Resource |
|-------|-----------------|
| Prompt Injection | [Perez & Ribeiro, 2022](https://arxiv.org/abs/2211.09527) |
| Prompt Injection (taxonomy) | [Liu et al., 2023](https://arxiv.org/abs/2310.12815) |
| Jailbreaking | [Wei et al., 2023](https://arxiv.org/abs/2307.02483) |
| Wild Jailbreaks | [Shen et al., 2023](https://arxiv.org/abs/2308.03825) |
| Adversarial Suffixes | [Zou et al., 2023](https://arxiv.org/abs/2307.15043) |
| Many-Shot | [Anthropic, 2024](https://www.anthropic.com/research/many-shot-jailbreaking) |
| Indirect Injection | [Greshake et al., 2023](https://arxiv.org/abs/2302.12173) |
| **OWASP Top 10 for LLM** | [owasp.org](https://owasp.org/www-project-top-10-for-large-language-model-applications/) |
| **MITRE ATLAS** | [atlas.mitre.org](https://atlas.mitre.org/) |

---

## Ethical Use

This tool is for **authorized security testing and education only**.

- Always obtain explicit permission before testing any system
- API keys belong in `.env` — never commit them
- Results may contain sensitive model outputs — handle accordingly
- The authors assume no liability for misuse

---

## Development Rules

See `docs/Rules.md` (gitignored, local only).

Quick summary:
- Every public function has a doc comment
- Every module has a `//!` module comment  
- No `unwrap()` in production paths
- New attack = code + TOML + educational content
- Commit style: `feat(attacks): add new payload set`
