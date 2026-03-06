# Architect Procedure

## Goal
Translate requirements into a boundary-safe design and explicit split/no-split decision.

## Prompt Shape
- Provide the current `requirements-spec.json` and incoming handoff.
- Ask for public interface changes, boundary impacts, policy/workflow implications, and child work item recommendations when needed.

## Required Output
- `architecture-design.json`
- `handoffs/architecture-design-to-implementation-plan.json`

## Guardrails
- Respect authoritative ownership: `enterprise` for ontology/policy, `schemas` for contracts, `workflows` for mutation paths, `services` for implementation.
- Prefer additive contract evolution.
- If the work item is not leaf-sized, stop at decomposition and do not define a code batch.
