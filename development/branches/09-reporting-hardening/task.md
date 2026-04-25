# task.md

## Branch

`codex/reporting-hardening`

## Wave

`Wave 6`, строго после `08-multi-turn-foundation`.

## Mission

Довести reporting model до состояния, пригодного для статьи, review и сравнения прогонов.

## Dependencies

- базироваться на `codex/weekend-integration` после merge ветки `08-multi-turn-foundation`.

## Sources Of Truth

- `Branch_tasks.md`
- `Roadmap_weekend.md`
- `TZ.md`
- `README.md`
- `refactoring.md`

## Allowed Scope

- `src/reporting/*`
- `src/engine/session.rs`
- `src/engine/damage.rs`
- `src/engine/evaluator.rs` только если это необходимо для согласованности evidence metadata
- docs по reports и review

## Forbidden Scope

- provider layer
- web-target policy
- scenario builder logic, кроме required metadata plumbing

## Required Outcomes

- summary и derived metrics имеют один источник истины;
- terminal и JSON reports расширены под scenario envelope, HTTP target, tool decisions, redaction и multi-turn transcripts;
- review и compare остаются удобными.

## Mandatory Checks

- `cargo check --all-targets`
- `cargo test`
- ручной review обычной CLI session
- ручной review scenario session
- ручной review HTTP target session
- ручной review multi-turn session

## Handoff

- перечислить изменённые reporting-файлы;
- описать новый report contract;
- перечислить проверки;
- указать формат новых metadata-полей;
- указать остаточные риски сравнения и review.

## Stop And Escalate If

- для завершения задачи требуется менять provider layer или web-target policy;
- reporting consistency нельзя достигнуть без пересмотра предыдущего этапа.
