# Desktop Snapshot Migration Note

- Durable desktop state in namespace `system.desktop` is authoritative for boot restore.
- Legacy browser snapshot data is compatibility input only.
- When both durable and legacy state exist, only the durable snapshot is eligible for restore.
- When only legacy state exists, the runtime migrates it into durable storage once and then treats that migrated representation as authoritative for any restore in the same session.
- When `restore_on_boot` is disabled, the runtime still performs the one-way legacy-to-durable migration when legacy data exists, but it does not hydrate layout from either source.
- Migration is deterministic and repeat-safe:
  - it never prefers legacy over an existing durable snapshot;
  - it only writes the typed durable `system.desktop` envelope;
  - rerunning the migration path with unchanged legacy input yields the same durable payload shape, with only the monotonic envelope timestamp differing.
