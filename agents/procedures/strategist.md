# Strategist Procedure

## Goal
Convert research and high-level intent into `research-synthesis.json` and `requirements-spec.json`.

For read-only governed decision support, the strategist may also assemble a `DecisionContextV1`
request for the `decision_evaluation` workflow when strategic or operational alternatives need
quantitative ranking under explicit risk and policy constraints.

## Prompt Shape
- State the work item id, parent id, iteration, objective, and affected modules.
- List the repo sources, policies, ontology files, ADRs, and prior work items being used.
- Ask for a repo-grounded synthesis, not speculation.

## Required Output
- `research-synthesis.json` with findings, constraints, source refs, and decomposition signals.
- `handoffs/research-synthesis-to-requirements-spec.json`
- `requirements-spec.json` with functional requirements, non-functional requirements, success metrics, assumptions, and out-of-scope items.
- `handoffs/requirements-spec-to-architecture-design.json`

## Guardrails
- Do not invent contracts or runtime behavior that is not grounded in the repo or cited sources.
- Mark unresolved decisions in `open_questions`.
- Recommend split when multiple module types, policy owners, or public interfaces are involved.
- Treat `decision_evaluation` as advisory-only; it cannot bypass workflow or policy gates.
