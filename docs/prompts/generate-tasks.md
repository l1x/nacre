<!--file:generate-tasks.md-->

# Generate Tasks

## Objective

Decompose a PRD into granular, actionable tickets with hierarchical structure suitable for iterative implementation.

## Input

- PRD file: `prds/prd-[feature-name]-[YYYY-MM-DD].md`

## Process

1. **Parse PRD**: Extract functional requirements, non-functional requirements, and acceptance criteria.
1. **Identify Epic**: Create one or multiple epics per PRD representing the full feature scope.
1. **Decompose into Tasks**

- Execute `bd prime` for ticket handling
- Break each requirement into atomic units of work (completable in one PR) that must have verifiable completion criteria derived from the PRD.

1. **Create Tickets**: Output tickets using `bd create`.

## Ticket Hierarchy

```
bd-a3f8        Epic: Feature scope (maps to PRD)
├── bd-a3f8.1      Task: Logical grouping of related work (PR)
└── bd-a3f8.2      Task: Logical grouping of related work (PR)
```

## Ticket Structure

Each ticket must include:

```
ID: bd-xxxx[.x]
Title: [Action verb] + [Component/Feature]
Type: Epic | Task
Parent: [Parent ID or none for Epic]
Status: backlog
Description: [What needs to be done]
Acceptance Criteria:
  - [ ] [Verifiable condition 1]
  - [ ] [Verifiable condition 2]
Dependencies: [List of blocking ticket IDs or none]
```

## Task Granularity Rules

- **Epic** — Full feature, multiple tasks, weeks of work
- **Task** — Atomic unit, one PR, hours of work (max 1 day)

## Output

- **Tool:** `bd prime`
- **Format:** Tickets created in Beads with hierarchical IDs

## Constraints

- Each task must be completable in a single PR
- Tasks must be independently verifiable
- Dependencies must be explicit
- Do NOT skip acceptance criteria
