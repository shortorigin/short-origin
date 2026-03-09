# First-Principles Implementation Roadmap

This roadmap converts the first-principles architecture baseline into a deterministic delivery
sequence. No later phase should begin until the prior phase exit criteria are satisfied.

## Charter

The roadmap exists to keep Origin aligned to the adopted architecture rather than drifting into
feature-by-feature exceptions. The platform must remain a governed enterprise substrate where
events, workflows, contracts, policies, and evaluation thresholds are all versioned control
artifacts.

## Global Invariants

The roadmap operationalizes the eight invariants defined in the
[first-principles baseline](first-principles-systems-architecture-report.md):

1. No production automation exists outside a workflow instance.
2. No workflow executes without emitting canonical events.
3. No privileged action occurs without identity, policy evaluation, and audit.
4. No cross-domain data exchange occurs without a versioned contract.
5. No agent acts directly on systems of record except through typed tools.
6. No autonomy tier increases without evaluation evidence, SLO compliance, and budget compliance.
7. No platform change is authoritative unless versioned through Git-reviewed control artifacts.
8. Hard state lives in governed stores; compute is replaceable, horizontally scalable, and
   observable.

## Phase 0: Governance Ratification and Structural Setup

**Objective**

Create the governance scaffolding required to enforce the ADR corpus.

**ADR dependencies**

- ADR-0005
- ADR-0013
- ADR-0015

**Tasks**

1. Ratify the adopted ADR corpus in version control.
2. Define the domain map and platform product-line ownership registry.
3. Create or refine control-artifact locations for infrastructure, workflows, policies, and
   evaluation configuration.
4. Define architecture review rules for cross-domain versus local changes.
5. Define the risk, consistency, and autonomy taxonomies.

**Deliverables**

- adopted ADR corpus
- domain ownership map
- platform ownership map
- classification standards for risk, consistency, and autonomy

**Exit criteria**

- every platform or domain area has an owner,
- the review process is documented and executable,
- no platform work proceeds without ADR references.

## Phase 1: Truth and Trace Foundations

**Objective**

Stand up the canonical event substrate and observability spine.

**ADR dependencies**

- ADR-0006
- ADR-0007
- ADR-0014
- ADR-0017

**Tasks**

1. Define the canonical event envelope and versioning rules.
2. Implement correlation and causation propagation.
3. Publish the operation criticality matrix and tag pilot workflows.
4. Build event ingestion and the authoritative event store.
5. Create the first projection service pattern.
6. Implement telemetry schema for traces, logs, and metrics.
7. Publish the initial service and workflow SLO catalog.

**Deliverables**

- canonical event specification
- event store and ingestion path
- projection framework
- trace propagation standard
- initial SLO set

**Exit criteria**

- a workflow event can be emitted, stored, projected, and traced end to end,
- every pilot operation has a declared consistency class,
- telemetry is visible across at least one end-to-end path.

## Phase 2: Contracts and Data Product Foundations

**Objective**

Make inter-domain data exchange testable and governed.

**ADR dependencies**

- ADR-0008
- ADR-0009

**Tasks**

1. Define the data contract specification.
2. Implement the contract registry and CI compatibility checks.
3. Select open analytical storage and table standards.
4. Create event-to-analytical landing pipelines.
5. Publish the first pilot domain data product contract.
6. Define lineage from canonical events to analytical tables and retrieval corpora.

**Deliverables**

- contract specification
- contract registry
- CI contract gates
- analytical landing zone
- first versioned domain data product

**Exit criteria**

- at least one domain publishes a versioned product contract,
- breaking contract changes fail CI,
- event lineage to analytics is traceable.

## Phase 3: Durable Workflow Execution Plane

**Objective**

Make enterprise actions reliable under retries, failures, and approvals.

**ADR dependencies**

- ADR-0007
- ADR-0010
- ADR-0014

**Tasks**

1. Select and instantiate the workflow runtime.
2. Publish workflow development standards for determinism, retries, timeouts, approvals, and
   compensations.
3. Implement the workflow metadata schema.
4. Build the approval-gate service or integration.
5. Implement saga and compensation support.
6. Migrate the first pilot automation into durable workflow form.

**Deliverables**

- workflow runtime
- workflow standards
- approval-gate mechanism
- compensation framework
- first production-like durable workflow

**Exit criteria**

- the pilot workflow survives a forced failure and resumes correctly,
- idempotent retry behavior is demonstrated,
- a human approval gate operates for the defined risk tier.

## Phase 4: Tooling and AI Execution Boundary

**Objective**

Constrain AI to planning and mediated action.

**ADR dependencies**

- ADR-0010
- ADR-0011
- ADR-0013

**Tasks**

1. Define the typed tool contract standard.
2. Implement a tool registry with auth scopes, side-effect classes, and audit requirements.
3. Add policy checks to the tool invocation path.
4. Require workflow binding for all side-effecting tools.
5. Build a planner runtime that cannot directly mutate systems of record.
6. Implement provenance logging for plan, evidence, tool request, and result.

**Deliverables**

- typed tool specification
- tool registry
- policy-enforced tool path
- planner runtime boundary
- AI provenance record model

**Exit criteria**

- AI completes pilot plans only through typed tools,
- direct side effects without workflow binding are technically impossible,
- every tool call has policy and audit records.

## Phase 5: Retrieval and Evaluation Stack

**Objective**

Operationalize grounded intelligence with replaceable retrieval infrastructure.

**ADR dependencies**

- ADR-0012
- ADR-0013
- ADR-0016

**Tasks**

1. Define the retrieval API contract with authorization and provenance.
2. Separate document-corpus ingestion from operational-state retrieval.
3. Stand up the first index backend behind the retrieval facade.
4. Define evaluation suites for retrieval, grounded answers, tool selection, and policy
   compliance.
5. Version prompts, retrieval configuration, and evaluation thresholds together.
6. Implement drift and regression monitoring.

**Deliverables**

- retrieval API
- corpus ingestion pipeline
- initial index backend
- evaluation harness
- versioned prompt, retrieval, and evaluation artifacts

**Exit criteria**

- retrieval consumers depend only on the API contract,
- evaluation runs in CI and pre-release checks,
- every retrieval response includes provenance.

## Phase 6: Control Plane Completion

**Objective**

Make policy, identity, secrets, and AI governance runtime-enforced across the platform.

**ADR dependencies**

- ADR-0013
- ADR-0015

**Tasks**

1. Implement the policy decision service and enforcement points.
2. Introduce workload identity for services, workflows, and agents.
3. Integrate secrets and key management.
4. Define risk-tier policies for tools and workflows.
5. Add policy-as-code review, test, promotion, and rollback.
6. Log every privileged action and policy decision to immutable audit streams.

**Deliverables**

- policy engine
- workload identity model
- secrets and key integration
- risk-tier policy packs
- immutable audit stream

**Exit criteria**

- no privileged tool call bypasses policy evaluation,
- policy changes are Git-reviewed and rollback-capable,
- audit completeness is demonstrated end to end for one pilot workflow.

## Phase 7: SRE and FinOps Runtime Governance

**Objective**

Use reliability and cost as dynamic autonomy controls.

**ADR dependencies**

- ADR-0014
- ADR-0016

**Tasks**

1. Instrument per-workflow and per-tool cost attribution.
2. Build dashboards for cost per completion, latency, rollback rate, human intervention rate, and
   policy denials.
3. Define error budgets for services and automation outcomes.
4. Define budget-burn actions for autonomy throttling, fallback, and review escalation.
5. Implement alerting and automated governance actions.

**Deliverables**

- unit-economics dashboard
- workflow reliability dashboard
- error-budget policy
- automated budget-response controls

**Exit criteria**

- autonomy throttling can be triggered automatically by budget burn,
- unit economics are visible per workflow family,
- cost and reliability signals are tied to release and runtime controls.

## Phase 8: Pilot Domain Productionization

**Objective**

Prove the architecture in one domain before expansion.

**ADR dependencies**

- all prior first-principles ADRs as applicable

**Tasks**

1. Select a low-to-medium-risk pilot domain.
2. Publish its vocabulary, events, contracts, workflows, tools, retrieval corpus, and evaluation
   suite.
3. Run a controlled pilot with human oversight.
4. Measure completion quality, rollback rate, review rate, SLO attainment, and unit economics.
5. Write corrective ADR amendments only when pilot evidence invalidates an adopted assumption.

**Deliverables**

- fully governed pilot domain
- pilot metrics report
- amendment ADRs if needed

**Exit criteria**

- the pilot meets safety, cost, and reliability thresholds,
- second-domain expansion criteria are explicit,
- the pilot implementation has no unresolved architectural bypasses.

## Phase 9: Multi-Domain Scale-Out

**Objective**

Expand without architectural drift.

**ADR dependencies**

- the full first-principles ADR corpus

**Tasks**

1. Onboard additional domains only through a standard onboarding template.
2. Reuse platform product lines instead of duplicating control logic.
3. Expand the contract registry, retrieval scope, and workflow catalogs incrementally.
4. Add process-mining and conformance analytics over canonical events.
5. Periodically remeasure retrieval, workflow, model, and cost behavior.

**Deliverables**

- second and third domain onboarding packages
- process intelligence dashboards
- periodic architecture compliance report

**Exit criteria**

- new domains integrate through standards rather than exceptions,
- cross-domain interoperability depends on contracts and events rather than hidden state,
- compliance reports show no unresolved ADR violations.

## Traceability Rules

Every implementation task, pull request, workflow definition, policy update, and contract change
that claims alignment with the baseline must include:

- one or more ADR IDs,
- impacted domains,
- affected consistency class,
- affected risk tier,
- rollback path,
- validation artifacts.

Those fields are mandatory because they turn the ADR set into an operating control system instead of
a static document set.
