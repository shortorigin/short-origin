# Issue Architecture Review 2026-03

Reviewed the repository's open issue set on March 9, 2026 against the current architecture source of truth:

- [`docs/adr/0001-sovereign-institutional-architecture.md`](../adr/0001-sovereign-institutional-architecture.md)
- [`docs/adr/0002-trust-zones-and-evidence.md`](../adr/0002-trust-zones-and-evidence.md)
- [`docs/adr/0004-wasmcloud-first-ui-shell.md`](../adr/0004-wasmcloud-first-ui-shell.md)
- [`docs/architecture/layer-boundaries.md`](../architecture/layer-boundaries.md)
- [`docs/architecture/runtime-composition.md`](../architecture/runtime-composition.md)
- [`ui/README.md`](../../ui/README.md)
- accepted UI ADRs under [`ui/docs/adr/`](../../ui/docs/adr)

Issue [`#102`](https://github.com/shortorigin/origin/issues/102) was opened to track the reconciliation work itself.

## Outcome

- 9 open issues remained architecturally relevant and were rewritten in place to the current subsystem and boundary model.
- 4 open issues were closed because they were completed on current `main`, duplicate, or superseded by narrower successor scope.
- Every surviving reviewed issue was linked to the `Engineering Flow` project and set to `Backlog`.

## Review Matrix

| Issue | Relevance Classification | Owning Subsystem | Architectural References | GitHub Action Taken |
| --- | --- | --- | --- | --- |
| [`#93`](https://github.com/shortorigin/origin/issues/93) | Active, aligned after rewrite | `xtask` delivery/governance validation; `.github/workflows`; `infrastructure/wasmcloud/manifests` | ADR 0002, ADR 0004, Runtime Composition and Delivery | Rewrote body in current architecture terms, added `type:infra` + `priority:medium`, linked to project, set `Backlog` |
| [`#78`](https://github.com/shortorigin/origin/issues/78) | Superseded umbrella issue | Residual scope redistributed to `#93`, `#73`, and `#39` | Successor issues now carry the governing ADR links | Added supersession comment and closed as `not planned` |
| [`#73`](https://github.com/shortorigin/origin/issues/73) | Active, aligned after rewrite | `shared/error-model`; `shared/telemetry`; `ui/crates/platform_host`; `ui/crates/desktop_runtime`; `ui/crates/site`; `ui/crates/desktop_tauri` | ADR 0001, ADR 0004, `ui/README.md`, Layer Boundaries | Rewrote body in place, merged overlapping scope from `#68`, kept `type:refactor` + `priority:medium`, set `Backlog` |
| [`#68`](https://github.com/shortorigin/origin/issues/68) | Duplicate/superseded | Scope absorbed into `#73` | Successor issue `#73` now carries ADR 0001 and ADR 0004 linkage | Added supersession comment and closed as `not planned` |
| [`#57`](https://github.com/shortorigin/origin/issues/57) | Completed on current `main` | Not applicable | Verification only: `cargo fmt --all --check` passed on March 9, 2026 | Added completion comment and closed as `completed` |
| [`#53`](https://github.com/shortorigin/origin/issues/53) | Completed on current `main`, with narrow residual scope | Residual parity scope retained in `#40` | Current browser/PWA parity follow-up moved to `#40` under ADR 0004 | Added completion/supersession comment and closed as `completed` |
| [`#40`](https://github.com/shortorigin/origin/issues/40) | Active, aligned after rewrite | `ui/crates/site`; `ui/crates/desktop_runtime` deep-link boundary | ADR 0004, WASM Target Architecture, `ui/README.md` | Rewrote body to the browser/PWA-baseline model, kept `type:refactor` + `priority:medium`, set `Backlog` |
| [`#39`](https://github.com/shortorigin/origin/issues/39) | Active, aligned after rewrite | `ui/crates/desktop_runtime::host`; `ui/crates/desktop_runtime` effect execution modules | ADR 0004, UI reducer-foundation ADR, `ui/README.md` | Rewrote body in place, kept `type:refactor` + `priority:medium`, set `Backlog` |
| [`#38`](https://github.com/shortorigin/origin/issues/38) | Active, aligned after rewrite | `ui/crates/desktop_runtime` shell orchestration modules | ADR 0004, UI reducer-foundation ADR, DOM-first compositor ADR, `ui/README.md` | Rewrote body in place, kept `type:refactor` + `priority:medium`, set `Backlog` |
| [`#37`](https://github.com/shortorigin/origin/issues/37) | Active, aligned after rewrite | `ui/crates/desktop_runtime::origin_wm`; `ui/crates/desktop_runtime::runtime_context` | ADR 0004, UI reducer-foundation ADR, `ui/README.md` | Rewrote body in place, kept `type:refactor` + `priority:medium`, set `Backlog` |
| [`#36`](https://github.com/shortorigin/origin/issues/36) | Active, aligned after rewrite | `ui/crates/desktop_runtime::origin_wm` | ADR 0004, UI reducer-foundation ADR, `ui/README.md` | Rewrote body and title to reflect continued decomposition on current `main`, kept `type:refactor` + `priority:medium`, set `Backlog` |
| [`#35`](https://github.com/shortorigin/origin/issues/35) | Active, aligned after rewrite | `ui/crates/system_ui`; `ui/crates/desktop_runtime` | ADR 0004, DOM-first compositor ADR, `ui/README.md` | Rewrote body in place, kept `type:refactor` + `priority:medium`, set `Backlog` |
| [`#34`](https://github.com/shortorigin/origin/issues/34) | Active, aligned after rewrite | `ui/crates/system_ui` | ADR 0004, `ui/README.md` | Rewrote body in place, kept `type:refactor` + `priority:medium`, set `Backlog` |

## Notes

- The surviving UI issues now name the exact `ui/` subsystem responsible for implementation and explicitly rule out leakage into service, workflow, infrastructure, or direct data-layer code.
- The surviving infrastructure issue now targets delivery validation, workflow automation, and manifest governance rather than open-ended CI or deployment work.
- The shared observability/error work now has one canonical issue (`#73`) instead of competing issue statements.
