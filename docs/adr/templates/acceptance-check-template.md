# Acceptance Check Template

Use this checklist when drafting the `Acceptance Checks` section of an ADR or when translating an
ADR into issue or PR acceptance criteria.

## Checklist

- [ ] The decision has at least one concrete verification statement tied to a repository artifact,
      runtime behavior, or governance control.
- [ ] The verification statement is observable in review, CI, or production evidence.
- [ ] The check states what must remain true, not just what should be implemented.
- [ ] The check names the affected surface: contract, workflow, policy, event, store, UI, or
      validation command.
- [ ] Multi-plane or risky decisions include rollback-sensitive checks.
- [ ] The acceptance checks do not duplicate the traceability fields; they verify the decision's
      effect.

## Example

```md
## Acceptance Checks

- Every side-effecting workflow records a workflow ID, actor, policy decision, and evidence
  manifest before completion.
- CI fails if a PR that changes a governed workflow omits its ADR reference or rollback path.
- The runtime can replay the workflow history without reissuing a non-idempotent external action.
```
