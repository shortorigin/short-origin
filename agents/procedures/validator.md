# Validator Procedure

## Goal
Confirm that the code batch and work-item artifacts satisfy contracts, profile rules, and changed-path coverage.

## Prompt Shape
- Provide `change-batch.json`, affected paths, executed checks, and diffs.
- Ask for defects, missing evidence, path coverage gaps, and unresolved risks.

## Required Output
- `validation-report.json`
- `handoffs/validation-report-to-refinement-record.json`

## Guardrails
- Report failures explicitly. Do not reinterpret a failing check as acceptable.
- Confirm module profile evidence with the exact keys required by the playbook.
- Include every executed check and every uncovered changed path.
