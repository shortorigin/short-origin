# UI System Refactor Issue Drafts

This document operationalizes the current UI review into ready-to-file GitHub issues. Each draft aligns to the repository's refactor issue form in [`.github/ISSUE_TEMPLATE/refactor.yml`](../../.github/ISSUE_TEMPLATE/refactor.yml) and is intended to be copied into GitHub with minimal editing.

Issue sequencing:

1. semantics substrate
2. overlay substrate
3. reducer modularization
4. dispatch performance
5. window orchestration extraction
6. effect pipeline formalization
7. web parity cleanup

## Issue 1

**Title**: `[Refactor]: Establish an accessibility-complete shared semantics substrate in system_ui`

**Summary**

Standardize ARIA and interaction semantics in `system_ui` primitives so tabs, menus, disclosures, and dialogs are correct by default and app crates stop re-implementing accessibility policy at the call site.

**Problem Statement**

`system_ui` already centralizes visual primitives, but it does not yet fully own the semantic contract for higher-level controls. Shared primitives expose some state attributes such as `aria-checked` and `aria-pressed`, while other required semantics are either missing or pushed into consumers. The most visible example is the shared tab primitive, which renders `role="tab"` without consistently providing the full tab-state contract. Menus and modals also rely on consumers to fill in parts of the accessibility pattern, which creates drift, duplicate logic, and inconsistent assistive-technology behavior across the shell.

This weakens the intended shared substrate model. A primitive layer that only standardizes markup shape and CSS hooks still leaves every app responsible for accessibility policy. That reduces cohesion, makes reviews harder, and increases the chance that behavior diverges across system apps.

**Why this matters now**

The repository has already committed to `ui/` as the owner of Leptos/Tauri presentation adapters and to `system_ui` as the reusable primitive layer. If semantic ownership remains split between primitives and consumers, every future shell/app refinement compounds the inconsistency. Fixing the substrate first makes later refactors cheaper, especially around overlays, dialogs, and navigation patterns.

**Relevant Context**

- Shared button semantics are defined in [controls.rs](../../ui/crates/system_ui/src/primitives/controls.rs).
- Shared navigation and tab primitives live in [navigation.rs](../../ui/crates/system_ui/src/primitives/navigation.rs).
- Shared menu and modal surfaces live in [overlays.rs](../../ui/crates/system_ui/src/primitives/overlays.rs).
- Current consumers in `desktop_runtime` compensate for primitive gaps with local keyboard/focus logic instead of receiving a complete substrate.

**Proposed Solution**

Expand `system_ui` so semantics are first-class primitive capabilities rather than ad hoc consumer concerns.

- Add missing semantic props to shared controls, including `aria-selected` and `aria-modal`, and make role-specific state wiring explicit at the primitive boundary.
- Upgrade `Tab`, `MenuItem`, `MenuSurface`, `DisclosurePanel`, and `Modal` so they encode the full expected ARIA pattern by default instead of delegating critical behavior to callers.
- Document which semantic guarantees primitives own and which responsibilities remain with feature-level containers.
- Add lightweight validation or targeted tests so primitives used with roles such as `tab`, `menuitemradio`, and `dialog` cannot silently omit required state.

**Acceptance Criteria**

- [ ] Shared primitives expose the semantic state needed for tabs, menus, disclosures, and dialogs without consumer patching.
- [ ] `Tab` instances correctly express active/inactive state to assistive technologies by default.
- [ ] Menu and modal primitives define the core WAI-ARIA surface expected by the desktop shell.
- [ ] System app crates no longer need to bolt on missing ARIA state for standard controls.
- [ ] Primitive-level accessibility tests cover the standardized semantics.

**Technical Notes**

- Preserve the current Rust-first ownership boundary: semantics stay in `ui/crates/system_ui`, while app-specific workflow logic remains in consumers.
- This issue should precede the overlay substrate refactor because overlay behavior should build on semantically complete primitives.

**Related Issues**

- Follow with the overlay interaction substrate issue in this document.
- Related paths: [controls.rs](../../ui/crates/system_ui/src/primitives/controls.rs), [navigation.rs](../../ui/crates/system_ui/src/primitives/navigation.rs), [overlays.rs](../../ui/crates/system_ui/src/primitives/overlays.rs)

## Issue 2

**Title**: `[Refactor]: Introduce a shared overlay interaction substrate for menus and popups`

**Summary**

Centralize overlay dismissal, focus restoration, roving focus, and pointer/keyboard parity so all shell menus and popups behave consistently across browser/WASM preview and desktop runtime hosts.

**Problem Statement**

Overlay behavior is currently distributed across taskbar code, menu components, and local accessibility helpers. Start menu, taskbar overflow, clock menu, and desktop context menu each carry their own combination of outside-click handling, escape handling, focus-first-item logic, and focus restoration. The shell also relies on `mousedown`-based global listeners in places where pointer-based behavior would be more robust across mouse, touch, and pen input. This creates duplicated logic, fragmented policy, and a high risk of subtle parity bugs.

The problem is architectural rather than cosmetic. Overlay interaction policy is a shared system concern, but it is currently implemented as local component behavior. That keeps the `system_ui` substrate thin and forces `desktop_runtime` composition code to own details that should be reusable.

**Why this matters now**

The overlay surfaces are a core part of the system UI shell. They are already numerous, and more will be added as settings, launcher, and system panels evolve. Without a shared substrate, every new overlay will copy existing patterns and widen the behavioral matrix that must be maintained.

**Relevant Context**

- Taskbar-level overlay listeners and focus orchestration live in [taskbar.rs](../../ui/crates/desktop_runtime/src/components/taskbar.rs).
- Menu-specific keyboard and close behavior is spread across [menus.rs](../../ui/crates/desktop_runtime/src/components/menus.rs).
- Local DOM focus helpers live in [a11y.rs](../../ui/crates/desktop_runtime/src/components/a11y.rs).

**Proposed Solution**

Create a shared overlay interaction layer that `desktop_runtime` can reuse across all shell surfaces.

- Introduce a reusable overlay controller or helper set that owns outside-pointer dismissal, escape handling, focus entry, roving menu navigation, and focus restoration to the invoking control.
- Standardize on pointer-aware outside interaction handling rather than a mix of local `mousedown` listeners and component-local stop-propagation rules.
- Define a single overlay lifecycle model for open, focus-first, navigate, dismiss, and restore-focus behavior.
- Keep `desktop_runtime` views responsible for declaring overlay state and invoking the shared substrate, not for re-implementing interaction policy.

**Acceptance Criteria**

- [ ] Start menu, overflow menu, clock menu, and desktop context menu use a shared overlay interaction substrate.
- [ ] Outside dismissal works consistently for mouse, touch, and pen input.
- [ ] Escape handling and focus restoration follow one policy across shell overlays.
- [ ] Menu keyboard navigation is shared rather than copied per menu.
- [ ] Browser/WASM parity tests cover at least one overlay dismissal flow and one keyboard navigation flow.

**Technical Notes**

- This should build on Issue 1 so overlay primitives already have complete semantics.
- Keep the host/runtime boundary unchanged; this is a UI substrate refactor, not a platform contract change.

**Related Issues**

- Depends on the semantic substrate issue in this document.
- Related paths: [taskbar.rs](../../ui/crates/desktop_runtime/src/components/taskbar.rs), [menus.rs](../../ui/crates/desktop_runtime/src/components/menus.rs), [a11y.rs](../../ui/crates/desktop_runtime/src/components/a11y.rs)

## Issue 3

**Title**: `[Refactor]: Split desktop_runtime reducer into domain reducers with typed effect builders`

**Summary**

Refactor the monolithic desktop reducer into focused Rust modules for window lifecycle, move/resize, appearance, app-command handling, and hydration/deep-link behavior while preserving the current authoritative transition model.

**Problem Statement**

`desktop_runtime/src/reducer.rs` has become the central execution point for nearly all shell state transitions and effect emission. That centralization is conceptually correct, but the current implementation shape is too broad. The file mixes action definitions, effect definitions, helper logic, domain transitions, recursive action dispatch, and a large test block in a single module. This makes invariants harder to audit, encourages duplication in effect emission, and raises the cost of modifying one domain without regressing another.

From a Rust maintenance perspective, the current reducer shape is no longer ergonomic. The code remains explicit, but its cohesion has degraded because unrelated domains share one implementation surface.

**Why this matters now**

This reducer is the shell's authoritative orchestration engine. As more features land, leaving it monolithic will slow reviews, make testing more brittle, and increase the chance of hidden coupling between appearance, app lifecycle, and window-management behavior.

**Relevant Context**

- The reducer currently spans the action/effect model and most transition logic in [reducer.rs](../../ui/crates/desktop_runtime/src/reducer.rs).
- Shared window-management helpers live in [window_manager.rs](../../ui/crates/desktop_runtime/src/window_manager.rs).

**Proposed Solution**

Retain the reducer as the authoritative boundary while modularizing its internal structure.

- Keep `DesktopAction` as the public dispatch surface initially to avoid breaking callers.
- Split handling into domain modules such as window lifecycle, move/resize, appearance/wallpaper, app commands, and hydration/deep links.
- Extract repeated lifecycle and effect-emission patterns into typed helpers or builders so focus transitions, persistence requests, and lifecycle events are declared consistently.
- Keep unit tests near their domain modules while preserving top-level integration tests for cross-domain transitions.

**Acceptance Criteria**

- [ ] The reducer is decomposed into smaller domain modules with explicit ownership boundaries.
- [ ] Public runtime contracts remain stable during the refactor.
- [ ] Repeated lifecycle/effect emission logic is centralized behind typed helpers.
- [ ] Existing reducer behavior remains covered by unit tests and cross-domain transition tests.
- [ ] Window-management invariants remain explicit and auditable after the split.

**Technical Notes**

- Preserve additive evolution: this issue should be behavior-preserving and avoid changing external action contracts unless a compatibility adapter is introduced.
- This refactor sets up Issue 4, Issue 5, and Issue 6 by making their target seams easier to isolate.

**Related Issues**

- Should precede dispatch dirty-tracking, orchestration extraction, and effect-pipeline formalization.
- Related paths: [reducer.rs](../../ui/crates/desktop_runtime/src/reducer.rs), [window_manager.rs](../../ui/crates/desktop_runtime/src/window_manager.rs)

## Issue 4

**Title**: `[Refactor]: Replace clone-on-dispatch state comparison with explicit dirty tracking`

**Summary**

Improve runtime dispatch performance and state clarity by removing full-state clone/equality comparison from the desktop provider and replacing it with explicit mutation results or dirty-slice tracking.

**Problem Statement**

`DesktopProvider` currently clones `DesktopState` and `InteractionState` on every dispatch, runs the reducer against mutable copies, then compares the entire post-reduction values against the pre-dispatch snapshots to decide which signals to update. This is straightforward, but it scales poorly as the shell state grows and as high-frequency interactions such as drag and resize produce repeated dispatches. More importantly, it makes correctness depend on whole-state equality rather than on explicit mutation intent.

This pattern is serviceable for small state containers, but it is not an ideal long-term Rust design for a shell runtime with frequent state churn. The provider should know exactly which slices changed because the reducer told it, not because it re-compared cloned snapshots after the fact.

**Why this matters now**

Window movement, resizing, and effect-heavy actions already put this path on the hot side of the runtime. Addressing the dispatch model early will make later orchestrator and reducer refactors less costly and provide a cleaner performance baseline for the shell.

**Relevant Context**

- Current dispatch orchestration and whole-state clone/equality comparison live in [runtime_context.rs](../../ui/crates/desktop_runtime/src/runtime_context.rs).

**Proposed Solution**

Make state mutation reporting explicit at the reducer/runtime boundary.

- Introduce a reducer result type that reports emitted effects plus which state domains were mutated.
- Update the provider to set only the signals whose domains were marked dirty.
- Preserve reducer explicitness and deterministic behavior while removing the need for whole-state equality as the update trigger.
- Add targeted performance regression coverage around multi-window actions, drag/resize loops, and effect-heavy action sequences.

**Acceptance Criteria**

- [ ] Dispatch no longer depends on cloning and comparing the full desktop and interaction states for correctness.
- [ ] The runtime updates only the signals marked dirty by the reducer result.
- [ ] Hot-path interactions such as move/resize remain behaviorally identical.
- [ ] Regression coverage exists for high-frequency dispatch scenarios.
- [ ] The new mechanism remains explicit, deterministic, and easy to audit in Rust.

**Technical Notes**

- This should follow reducer modularization so the dirty-state model can be introduced cleanly across domain reducers.
- Favor a typed Rust result over implicit interior mutation tracking.

**Related Issues**

- Depends on the reducer modularization issue in this document.
- Related path: [runtime_context.rs](../../ui/crates/desktop_runtime/src/runtime_context.rs)

## Issue 5

**Title**: `[Refactor]: Extract a typed window orchestration layer from desktop shell views`

**Summary**

Move window/session orchestration policy out of Leptos view composition and into a typed Rust layer that owns focus, placement, restore/minimize behavior, and preferred-window selection.

**Problem Statement**

`desktop_runtime` composition code currently does more than translate DOM events into reducer actions. It also participates in orchestration decisions such as opening settings, choosing preferred windows, focusing or restoring existing app instances, and coordinating some shell-level behaviors across window and taskbar components. This blurs the line between presentation and orchestration and makes policy harder to test without mounting UI components.

The architecture wants Rust-owned, explicit orchestration boundaries. Right now the views are still carrying part of that policy surface.

**Why this matters now**

As the shell grows, keeping orchestration logic in view modules will create duplicate policy, harder-to-test behavior, and weaker separation between presentation and system runtime concerns. Extracting a typed layer now improves cohesion and makes later refactors around reducer structure and effect execution safer.

**Relevant Context**

- Shell composition and helper orchestration live in [components.rs](../../ui/crates/desktop_runtime/src/components.rs).
- Window interaction view logic lives in [window.rs](../../ui/crates/desktop_runtime/src/components/window.rs).
- Related orchestration wiring also touches taskbar behavior in [taskbar.rs](../../ui/crates/desktop_runtime/src/components/taskbar.rs).

**Proposed Solution**

Introduce a typed orchestration layer that sits between views and reducer dispatch.

- Define a reusable Rust module or service that owns preferred-window lookup, focus-or-restore behavior, default placement policy, snap/maximize policy, and app/window activation rules.
- Keep Leptos components responsible for translating DOM and pointer events into typed orchestration calls or direct reducer actions, not for making policy decisions.
- Move test coverage for orchestration rules into Rust unit tests that do not require mounted UI components.

**Acceptance Criteria**

- [ ] Window orchestration policy is concentrated in typed Rust modules rather than distributed across view files.
- [ ] View components become thinner and primarily translate user input into orchestration calls or actions.
- [ ] Preferred-window, focus/restore, and placement logic are testable without Leptos component mounting.
- [ ] Policy duplication between taskbar, window, and shell helpers is reduced.
- [ ] External runtime behavior remains stable while internal boundaries become clearer.

**Technical Notes**

- Keep orchestration in `ui/` and do not bypass typed app/runtime contracts.
- This issue should follow reducer modularization and can proceed in parallel with dispatch dirty tracking once the reducer seams are clearer.

**Related Issues**

- Builds on the reducer modularization issue.
- Related paths: [components.rs](../../ui/crates/desktop_runtime/src/components.rs), [window.rs](../../ui/crates/desktop_runtime/src/components/window.rs), [taskbar.rs](../../ui/crates/desktop_runtime/src/components/taskbar.rs)

## Issue 6

**Title**: `[Refactor]: Formalize runtime effect execution as a typed command pipeline`

**Summary**

Reshape runtime effect execution into explicit, domain-scoped handlers with clear ordering guarantees so host integration is easier to reason about, test, and evolve.

**Problem Statement**

The current effect system preserves an explicit reducer-to-host boundary, which is the right architectural direction. However, effect execution is still organized around a broad top-level queue plus a large `match` over `RuntimeEffect` variants. As the effect surface grows, this makes ordering, nested dispatch behavior, capability scoping, and testability harder to reason about. Host integration remains typed, but the effect pipeline itself is still structurally thin.

This is less a correctness bug than a maintainability risk. The pipeline has the right conceptual boundary but not yet the right internal shape for long-term growth.

**Why this matters now**

The effect pipeline is the place where UI orchestration meets host services, persistence, notifications, app bus behavior, and browser integration. A more structured pipeline will make later refactors safer and will give the system a better place to express deterministic ordering and telemetry.

**Relevant Context**

- Queue draining is installed in [effect_executor.rs](../../ui/crates/desktop_runtime/src/effect_executor.rs).
- Effect-to-host dispatch is centralized in [host/effects.rs](../../ui/crates/desktop_runtime/src/host/effects.rs).
- The host boundary is defined in [host.rs](../../ui/crates/desktop_runtime/src/host.rs).

**Proposed Solution**

Formalize effect execution as a typed command pipeline with domain-scoped handlers.

- Group effects by capability area such as host UI, persistence, wallpaper, and app bus execution.
- Make ordering guarantees explicit, including how nested dispatches enqueue follow-on work.
- Add test hooks or lightweight telemetry so effect execution can be asserted without depending on mounted UI composition.
- Preserve the current reducer/effect separation while making handler ownership more modular and auditable.

**Acceptance Criteria**

- [ ] Effect execution is organized into domain-scoped handlers rather than one broad dispatcher.
- [ ] Ordering guarantees for queue draining and nested dispatch are explicit and tested.
- [ ] Host integration remains behind typed Rust boundaries.
- [ ] Effect execution can be unit-tested without full UI mounting.
- [ ] Telemetry or test hooks exist for observing executed effect sequences.

**Technical Notes**

- This issue should follow reducer modularization so the new effect builder patterns and handler boundaries align.
- Avoid changing host capability contracts unless a compatibility-preserving adapter is provided.

**Related Issues**

- Closely related to reducer modularization and window orchestration extraction.
- Related paths: [effect_executor.rs](../../ui/crates/desktop_runtime/src/effect_executor.rs), [effects.rs](../../ui/crates/desktop_runtime/src/host/effects.rs), [host.rs](../../ui/crates/desktop_runtime/src/host.rs)

## Issue 7

**Title**: `[Refactor]: Close browser and desktop parity gaps in deep links and canonical shell entry flows`

**Summary**

Align the web-facing shell entrypoints with the desktop runtime contract by fixing deep-link encoding/decoding, clarifying ownership of deep-link boot behavior, and replacing placeholder canonical routes with governed read-only behavior.

**Problem Statement**

The site shell currently mixes real deep-link bootstrap behavior with placeholder canonical routes. Query/hash parsing does not consistently account for URL encoding and decoding, and deep-link application remains close to view-layer boot code rather than a clearly isolated routing/deep-link boundary. This makes browser/desktop parity weaker than intended and leaves a gap between the canonical web entrypoints and the runtime-owned desktop experience.

The issue is architectural as much as functional: the browser path should be another governed entry surface into the same typed runtime concepts, not a partially separate experience with placeholder behavior and scattered parsing logic.

**Why this matters now**

The repository keeps browser/PWA support as the baseline runtime while using the Tauri desktop host
as a capability extension over the same shell. To make that stance credible, parity-critical
routing and deep-link behavior need to be explicit, tested, and free of placeholder drift.

**Relevant Context**

- Current route composition, canonical placeholders, and deep-link parsing live in [web_app.rs](../../ui/crates/site/src/web_app.rs).

**Proposed Solution**

Strengthen the routing/deep-link boundary and remove placeholder behavior that weakens parity.

- Apply URL encoding when generating deep-link URLs and decode query/hash values before parsing open targets.
- Extract deep-link parsing and idempotent application into a clearer routing/deep-link ownership boundary.
- Replace placeholder note/project routes with contract-backed read-only content or an explicit temporary state that is clearly governed and tested.
- Add typed tests for encoded values, hash/query variants, and repeated boot application behavior.

**Acceptance Criteria**

- [ ] Deep-link generation and parsing handle percent-encoded values correctly.
- [ ] Deep-link application is idempotent and covered by tests.
- [ ] Canonical note/project routes no longer ship as ungoverned placeholder content.
- [ ] Browser/WASM parity expectations are explicit and verified by tests.
- [ ] The routing/deep-link boundary is clearer than the current view-local bootstrap approach.

**Technical Notes**

- Keep the browser/PWA surface as the baseline runtime while preserving parity across the Tauri host.
- This issue can proceed after the core shell substrate issues because it depends more on routing ownership than on reducer internals.

**Related Issues**

- Related path: [web_app.rs](../../ui/crates/site/src/web_app.rs)
