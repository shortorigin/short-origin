# Implementation Planner Procedure

## Goal
Produce a decision-complete `implementation-plan.json` for either child slices or a leaf execution batch.

## Prompt Shape
- Provide `architecture-design.json`, affected paths, acceptance criteria, and required module profiles.
- Ask for concrete change slices, target paths, tests, rollout notes, and profile evidence keys.

## Required Output
- `implementation-plan.json`
- `handoffs/implementation-plan-to-change-batch.json` for leaf work items.
- `handoffs/implementation-plan-to-refinement-record.json` for parent work items that split.

## Guardrails
- A leaf plan must not leave interface or file-scope decisions open.
- Parent work items stop after planning and hand off to refinement with `decision = split`.
- Every touched module profile must be represented in `profile_evidence`.
