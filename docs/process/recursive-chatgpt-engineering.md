# Recursive ChatGPT Engineering

> Archived on 2026-03-05. This playbook is preserved as historical context for the retired `work-items/` process. Active delivery now follows [DEVELOPMENT_MODEL.md](../../DEVELOPMENT_MODEL.md) and the GitHub rollout runbook in [docs/process/github-governance-rollout.md](github-governance-rollout.md).

## Purpose
This playbook defines the required prompt-driven delivery loop for Short Origin:

`research -> requirement synthesis -> architecture -> implementation plan -> code generation batch -> validation -> refinement`

The loop is recursive. Run it at the system, domain, module, workflow/service pair, and leaf-change levels. Parent work items split into child work items when they cross multiple authoritative module types, policy owners, or public interfaces.

## Required Artifacts
Every material change is tracked under `work-items/<work-item-id>/`.

- `work-item.json`: scope, traceability, decomposition, and current state.
- `iteration-XX/research-synthesis.json`
- `iteration-XX/requirements-spec.json`
- `iteration-XX/architecture-design.json`
- `iteration-XX/implementation-plan.json`
- `iteration-XX/change-batch.json` for leaf work items only.
- `iteration-XX/validation-report.json` for leaf work items only.
- `iteration-XX/refinement-record.json`
- `iteration-XX/handoffs/<from>-to-<to>.json` for every adjacent stage transition that exists in the iteration.

All stage artifacts must carry:

- `work_item_id`
- `parent_work_item_id`
- `iteration`
- `affected_paths`
- `affected_modules`
- `policy_refs`
- `acceptance_criteria`
- `open_questions`
- `verification_status`

## Stage Contract
Each stage must produce outputs that become inputs to the next stage.

### Research
- Inputs: objective, upstream policies, ADRs, ontology, existing contracts, prior work items.
- Outputs: `research-synthesis.json`, including findings, constraints, source refs, and decomposition signals.
- Stop condition: enough repo-grounded evidence exists to write explicit requirements.

### Requirement Synthesis
- Inputs: research synthesis and incoming handoff.
- Outputs: `requirements-spec.json`, including functional requirements, non-functional requirements, success metrics, assumptions, and out-of-scope items.
- Stop condition: success can be evaluated without inventing new product intent mid-implementation.

### Architecture
- Inputs: requirements spec and incoming handoff.
- Outputs: `architecture-design.json`, including boundary impacts, public interface changes, and decomposition decision.
- Stop condition: boundary ownership, contract changes, and split/no-split decision are fixed.

### Implementation Plan
- Inputs: architecture design and incoming handoff.
- Outputs: `implementation-plan.json`, including change slices, target paths, test scenarios, rollout notes, and module profile evidence.
- Stop condition: a leaf implementer can execute without making design decisions.

### Code Generation Batch
- Inputs: implementation plan and incoming handoff.
- Outputs: `change-batch.json`, describing the exact bounded code batch for the leaf work item.
- Stop condition: the batch is reviewable in one PR and no high-severity open questions remain.

### Validation
- Inputs: change batch and incoming handoff.
- Outputs: `validation-report.json`, including checks run, findings, changed path coverage, and confirmed module profile evidence.
- Stop condition: defects are either resolved or explicitly returned to refinement.

### Refinement
- Inputs: validation report for leaf work items, or implementation plan for split parent work items.
- Outputs: `refinement-record.json`, including decision (`close`, `iterate`, or `split`), residual risks, improvements, and next work item ids.
- Stop condition: the iteration is either closed or the next iteration/work items are explicitly created.

## Recursion Rules
- Split after research or architecture when the change touches more than one authoritative module type, more than one policy owner, or more than one public contract/event surface.
- Parent work items stop at implementation planning and refinement. They do not emit `change-batch.json` or `validation-report.json`.
- Leaf work items must have fixed `acceptance_criteria`, fixed `affected_paths`, and no unresolved high-severity open questions before `change-batch.json` is allowed.

## Module Profiles
`implementation-plan.json` and `validation-report.json` must record profile evidence with the exact areas below when those modules are touched:

- `enterprise.ontology_alignment`
- `enterprise.downstream_impacts`
- `schemas.compatibility_notes`
- `schemas.fixture_updates`
- `services.contract_refs`
- `services.owned_aggregate_boundaries`
- `workflows.failure_modes`
- `workflows.compensations`
- `workflows.policy_gates`
- `workflows.audit_outputs`
- `platform.runtime_compatibility`
- `agents.deterministic_io`
- `agents.eval_updates`
- `infrastructure.policy_leakage_review`
- `shared.reuse_notes`
- `shared.policy_leakage_review`

## Verification Gates
- The Rust workspace gates remain mandatory.
- `cargo run -p process-method -- validate ...` is mandatory for every PR and push.
- Changed paths must be covered by a committed work item `affected_paths` or `target_paths`.
- Parent/child work item links, stage order, handoffs, and module profile evidence must all validate before Rust quality gates are considered sufficient.
