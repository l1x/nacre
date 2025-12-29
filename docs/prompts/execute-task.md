<!--file:execute-task.md-->

# Execute Task

## Objective

Implement a single, atomic task defined in a Beads ticket, ensuring code quality and verification before completion.

## Input

- **Ticket ID**: The `bd` ticket ID to execute (e.g., `bd-a3f8.1`).

## Process

1.  **Claim Task**
    - Read the task details: `bd show <id>`
    - Mark the task as in progress: `bd update <id> --status=in_progress`

2.  **Context Loading**
    - Read the parent Epic or PRD if referenced to understand the broader scope.
    - Explore relevant source code sections.
    - Ignore compiled or otherwised generated content.

3.  **Implementation**
    - **Create Test**: Write a failing test case that reproduces the requirement or bug.
    - **Implement**: Write the minimal code necessary to pass the test.
    - **Refactor**: Clean up code while keeping tests passing.

4.  **Verification**
    - Run formatting: `mise run fmt`
    - Run linting: `mise run lint`
    - Run tests: `mise run tests`
    - Run security audit (if applicable): `mise run audit`

5.  **Completion**
    - If verification fails, fix issues and repeat Step 4.
    - If verification passes:
      - Output a summary of changes made into the description of the task
      - Mark ticket blocked: `bd update <id> --status blocked`

## Available mise Tasks

**Code Quality:**

- `mise run fmt` - Format code with cargo fmt
- `mise run lint` - Lint with Clippy, failing on warnings
- `mise run tests` - Run all tests with output
- `mise run audit` - Run security audit on dependencies

**Building:**

- `mise run build-dev` - Build development version
- `mise run build-prod` - Build release version (includes frontend)
- `mise run build-js-prod` - Build frontend TypeScript only
- `mise run start-dev-server` - Run dev server with browser open

**Testing & Analysis:**

- `mise run coverage` - Run tests with coverage report
- `mise run coverage-html` - Generate HTML coverage report
- `mise run machete` - Find unused dependencies
- `mise run check-deps` - Run both audit and machete

**Project Management:**

- `mise run show-issues` - List all open issues
- `mise run show-ready` - Show ready work (no blockers)
- `mise run show-blocked` - Show blocked issues
- `mise run show-issue-stats` - Show issue statistics
- `mise run show-issue-tree` - Show dependency tree for issues

## Constraints

- **Scope**: Focus ONLY on the specified ticket. Do not implement extra features.
- **Quality**: Code must compile and pass all verification steps.
- **Atomic**: If the task is too large, stop and request to split it into smaller tickets.

## Error Handling

- If you encounter a blocker (missing dependency, unclear requirement), add a comment to the ticket (`bd comment <id> "Blocker: ..."`) and move the ticket to be blocked and stop.
