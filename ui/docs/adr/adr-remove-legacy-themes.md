---
adr_id: UI-ADR-REMOVE-LEGACY-THEMES
title: Remove Legacy Themes from Core Baseline
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - ui
impacted_domains:
  - ui-shell
source_report: null
roadmap_phases: []
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - ui
  - themes
  - accessibility
---

# ADR: Remove Legacy Themes from Core Baseline

## Context

Multiple baseline shell skins were creating styling fragmentation, maintenance cost, and inconsistent
component behavior in the primary runtime path.

## Decision

Legacy shell skins are removed from the primary runtime path. The shipping baseline is one
contemporary shell style with accessibility variants for high contrast and reduced motion.

## Constraints

- Accessibility variants remain mandatory.
- The core runtime path should present one coherent shell identity.
- Legacy theme support must not fragment baseline testing or behavior.

## Alternatives Considered

- `Keep all legacy skins in the default boot path`: rejected because it preserves styling
  fragmentation.
- `Preserve runtime skin switching behind compatibility flags`: rejected because it prolongs
  architectural drift.

## Rationale

One baseline shell style reduces maintenance burden and makes accessibility the primary variation
surface rather than nostalgia styling.

## Implementation Implications

- Legacy theme assets are archival or optional only.
- Theme state centers on accessibility flags instead of baseline style selection.
- Shell, apps, and tests assume a single baseline style identifier.

## Acceptance Checks

- The shipping runtime path exposes one default shell style.
- Accessibility variants remain supported without restoring legacy baseline skins.
- Tests and shell defaults stop assuming multiple equal-priority theme baselines.

## Traceability

- Governing source: UI-local adopted ADR under the repository-wide ADR contract
- Related roadmap phases: none
- Related controls: `docs/adr/README.md`, UI shell styling baseline
