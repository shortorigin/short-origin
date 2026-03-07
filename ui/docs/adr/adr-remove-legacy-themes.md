# ADR: Remove Legacy Themes from Core Baseline

## Status
Accepted

## Decision
Legacy shell skins are removed from the primary runtime path. The shipping baseline is one contemporary shell style with accessibility variants for high contrast and reduced motion.

## Rationale
- Multiple baseline skins created maintenance drag and inconsistent component behavior.
- The runtime should feel like one operating shell, not a collection of separately styled surfaces.
- Accessibility options remain important, but nostalgia skins are not required for the core product path.

## Alternatives Rejected
- Keep all legacy skins in the default boot path: rejected because it preserves styling fragmentation.
- Preserve runtime skin switching behind compatibility flags: rejected because it prolongs architectural drift.

## Consequences
- Legacy theme assets are archival or optional only.
- Theme state now centers on accessibility flags instead of style selection.
- Shell, apps, and tests should assume a single baseline style identifier.
