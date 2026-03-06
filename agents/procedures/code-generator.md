# Code Generator Procedure

## Goal
Generate one bounded, reviewable `change-batch.json` for a leaf work item.

## Prompt Shape
- Provide `implementation-plan.json`, exact target paths, acceptance criteria, and blocking questions.
- Ask for the smallest coherent code batch that satisfies the plan.

## Required Output
- `change-batch.json`
- `handoffs/change-batch-to-validation-report.json`

## Guardrails
- Only execute on leaf work items.
- Refuse to proceed if high-severity `open_questions` remain.
- Keep the batch within the paths and interfaces already approved by the plan.
