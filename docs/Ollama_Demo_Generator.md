# Ollama Demo Generator

## Scope

This workflow prepares an `Ollama` demo harness for this repository without changing `src/` or `payloads/`.

Fixed paths used by the generator:

- models: `E:\repos\AI-security-test\ollama_models`
- generated and auxiliary files: `E:\repos\AI-security-test\buffer_for_ollama`

## What The Scripts Do

Committed scripts:

- `scripts/Setup-OllamaDemo.ps1`
- `scripts/Run-OllamaScenarioMatrix.ps1`

Generated runtime files in `buffer_for_ollama/`:

- `start-ollama-server.ps1`
- `pull-models.ps1`
- `demo-commands.ps1`
- `ollama-demo.env`

## Why `OLLAMA_MODELS` Matters

If you want model blobs to live under the repository, `ollama pull` must run with:

```powershell
$env:OLLAMA_MODELS = 'E:\repos\AI-security-test\ollama_models'
```

Without that variable, Ollama stores models in its default global location instead of this repo.

## Setup

Generate the helper files:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\Setup-OllamaDemo.ps1
```

Generate the helper files and immediately pull the recommended models:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\Setup-OllamaDemo.ps1 -PullModels
```

Default demo model set:

- `llama3.1:8b`
- `qwen2.5:7b`
- `mistral:7b`

## Start Ollama With Repo Paths

Use the generated helper so the server sees the repository model directory:

```powershell
powershell -ExecutionPolicy Bypass -File .\buffer_for_ollama\start-ollama-server.ps1
```

If Ollama already runs as a background service, stop that instance first or make sure your interactive shell uses the same model store before pulling models.

## Pull Models Later

If you do not want to download models during setup:

```powershell
powershell -ExecutionPolicy Bypass -File .\buffer_for_ollama\pull-models.ps1
```

## Apply The `.env` Snippet

The setup script writes:

- `buffer_for_ollama\ollama-demo.env`

Recommended values:

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

Copy the values you need into the repository `.env`.

## Run The Demo Matrix

Run all three exposure scenarios across the selected models:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\Run-OllamaScenarioMatrix.ps1
```

Custom model list example:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\Run-OllamaScenarioMatrix.ps1 -Models llama3.1:8b,qwen2.5:7b -Limit 3
```

The script uses:

- `support_bot`
- `hr_bot`
- `internal_rag_bot`

For `internal_rag_bot` it automatically adds `--retrieval-mode subset`.

## Validation

After setup:

```powershell
$env:OLLAMA_MODELS = 'E:\repos\AI-security-test\ollama_models'
ollama list
cargo run --bin ai-sec -- check --provider ollama
```

## Notes

- the scripts do not modify Rust sources;
- the scripts do not modify `payloads/`;
- model downloads are opt-in via `-PullModels` or the generated `pull-models.ps1`;
- all generated helper files land in `buffer_for_ollama/`, which is gitignored.
