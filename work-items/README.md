# Work Items

Archived historical record. New work must be tracked in GitHub issues, pull requests, milestones, and the organization Project board described in [DEVELOPMENT_MODEL.md](/Users/justinshort/short%20origin/DEVELOPMENT_MODEL.md). Active long-task execution artifacts now live under [`plans/`](/Users/justinshort/short%20origin/plans/); they are companion artifacts only and do not replace the GitHub system of record.

The existing `work-items/` tree remains in git for traceability only. See [docs/process/work-item-migration.md](/Users/justinshort/short%20origin/docs/process/work-item-migration.md) for the cutover record.

## Layout
- `work-items/<work-item-id>/work-item.json`
- `work-items/<work-item-id>/iteration-XX/*.json`
- `work-items/<work-item-id>/iteration-XX/handoffs/*.json`

Parent work items stop after planning and refinement with `decision = "split"`.
Leaf work items continue through `change-batch.json`, `validation-report.json`, and `refinement-record.json`.

No new `work-items/` should be created.
