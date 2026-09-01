## Purpose

Automated dependency update policy for Rust, frontend, and CI dependencies.

## Requirements

### Requirement: Dependency Updates

Dependabot SHALL repeatedly check for updates to Cargo dependencies, npm dependencies, and GitHub Actions, using the `deps: ` commit prefix.

#### Scenario: Cargo dependency update available

- **WHEN** Dependabot finds a new version of a Cargo dependency during a scheduled check
- **THEN** it creates a PR with commit prefix `deps: `

#### Scenario: npm dependency update available

- **WHEN** Dependabot finds a new version of an npm dependency during a scheduled check
- **THEN** it creates a PR with commit prefix `deps: `

#### Scenario: GitHub Actions update available

- **WHEN** Dependabot finds a new version of a GitHub Action during a scheduled check
- **THEN** it creates a PR with commit prefix `deps: `
