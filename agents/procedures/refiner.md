# Refiner Procedure

## Goal
Close the loop by deciding whether the work item closes, iterates, or splits.

## Prompt Shape
- Provide the latest stage artifact and incoming handoff.
- Ask for residual risks, lessons learned, and the next work item ids when further work is required.

## Required Output
- `refinement-record.json`

## Guardrails
- Use `split` only for parent work items with child ids.
- Use `close` only when validation passed and no follow-on work item is required.
- Any additional work must be expressed as a new iteration or explicit child work item ids.
