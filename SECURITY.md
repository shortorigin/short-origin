# Security Policy

## Reporting Vulnerabilities

Do not open public GitHub issues for suspected vulnerabilities.

Use GitHub private vulnerability reporting when it is enabled for the repository. If private reporting is not available, contact the repository maintainers through the `shortorigin` organization before disclosing details publicly.

## Report Content

Include:

- affected repository and component
- impact summary
- reproduction steps or proof of concept
- known mitigations or workarounds
- any proposed fix direction

## Response Expectations

Maintainers will triage reports, determine severity, and coordinate remediation and disclosure timing. Security fixes should follow the normal PR review path once the advisory is ready for disclosure or the patch is safe to land publicly.

## Repository Security Controls

The public repository should enable:

- GitHub private vulnerability reporting
- secret scanning and push protection
- Dependabot alerts and automated update PRs
- CodeQL analysis for Rust and JavaScript/TypeScript surfaces
