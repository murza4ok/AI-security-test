# Test Instruction

## Purpose

This file is the operator-facing smoke checklist for the current repository
state.

It is intentionally narrower than `README.md`: use it when you want a practical
validation sequence instead of a full product walkthrough.

## Source Of Truth

When documents disagree, trust them in this order:

1. `development/STATUS.md`
2. `README.md`
3. `docs/HTTP_Target_Mode.md`
4. `docs/Ollama_Demo_Setup.md`
5. `Architecture.md` and `TZ.md`

## Preconditions

- current directory is the repository root;
- `ai-sec` is launched with `cargo run --bin ai-sec -- ...`;
- `web_target` is launched separately with `cargo run --bin web_target --`;
- at least one provider is configured, or you are using HTTP target mode;
- for `ollama`, verify that the local daemon and chosen model are available.

## Minimal CLI Validation

These commands do not require a live target application:

```bash
cargo run --bin ai-sec -- list
cargo run --bin ai-sec -- help run
```

If you use `ollama`, verify the provider contract explicitly:

```bash
cargo run --bin ai-sec -- check --provider ollama
```

## Direct Provider Smoke

Minimal direct attack run:

```bash
cargo run --bin ai-sec -- run --attack jailbreaking --provider ollama --limit 1 --output /tmp/ai-sec-direct.json
cargo run --bin ai-sec -- review /tmp/ai-sec-direct.json
```

## Scenario Smoke

Minimal scenario-driven run:

```bash
cargo run --bin ai-sec -- run --attack sensitive_data_exposure --provider ollama --app-scenario support_bot --limit 1 --output /tmp/ai-sec-scenario.json
cargo run --bin ai-sec -- review /tmp/ai-sec-scenario.json
```

Useful variants:

- `--app-scenario hr_bot`
- `--app-scenario internal_rag_bot --retrieval-mode subset`
- `--app-scenario support_bot_hardened`

## HTTP Target Smoke

Start the demo target in a separate terminal:

```bash
cargo run --bin web_target --
```

Optional health check:

```bash
curl -i http://127.0.0.1:3000/health
```

Then run an HTTP attack session:

```bash
cargo run --bin ai-sec -- run --attack jailbreaking --target-mode http --target-base-url http://127.0.0.1:3000 --target-user customer_alice --target-profile naive --limit 1 --output /tmp/ai-sec-http.json
cargo run --bin ai-sec -- review /tmp/ai-sec-http.json
```

## Multi-Turn Smoke

Minimal chain execution check:

```bash
cargo run --bin ai-sec -- run --attack prompt_injection --provider ollama --limit 1 --output /tmp/ai-sec-multiturn.json
cargo run --bin ai-sec -- review /tmp/ai-sec-multiturn.json
```

## Compare Smoke

Once you have at least two saved reports:

```bash
cargo run --bin ai-sec -- compare /tmp/ai-sec-direct.json /tmp/ai-sec-scenario.json /tmp/ai-sec-http.json /tmp/ai-sec-multiturn.json
```

Or compare everything from the default `results/` directory:

```bash
cargo run --bin ai-sec -- compare
```

## Expected Outcome

After this checklist you should have verified:

- CLI help and attack listing;
- at least one live provider path;
- one scenario report;
- one HTTP target report;
- one multi-turn report;
- readable `review` and `compare` output.

## What This File No Longer Assumes

The current project state does not require or guarantee:

- a `scripts/` directory with helper automation;
- fixed absolute repository paths;
- a `main`-branch-only workflow;
- PowerShell setup steps;
- repo-local `buffer_for_ollama/` artifacts.
