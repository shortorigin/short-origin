# ADR 0003: Capital Markets Integration

## Status
Accepted

## Decision
The standalone finance source tree is decomposed into the primary super-repository instead of being imported as a sibling workspace. Capital-markets ontology and service ownership live in `enterprise/`, versioned finance contracts and events live in `schemas/`, deterministic reusable trading logic lives in `shared/`, the Wasmtime-safe strategy runtime lives in `platform/runtime/strategy-sandbox`, stateful capital-markets capabilities live in dedicated services, and strategy promotion orchestration lives in `workflows/quant_strategy_promotion`.

## Consequences
- Trading portfolio, execution, research, market-data, and trading-risk aggregates remain separate bounded contexts and do not collapse into `finance-service`.
- Finance-specific envelope, logger, audit-trail, and controlplane orchestration duplicates are replaced by the repository’s canonical event, evidence, and workflow layers.
- Deterministic fixtures for the migrated pipeline are stored under `testing/fixtures/finance/` and validated by workflow tests inside the root workspace.
- New capital-markets changes must follow the root workspace gates, Rust 2021 compatibility, shared dependency inheritance, and schema-first interface evolution rules.
