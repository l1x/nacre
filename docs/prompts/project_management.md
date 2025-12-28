# Beads Project Management Issue Hierarchy & Dependencies

This guide explains how to structure work in beads using two complementary concepts.

## Core Concepts

| Concept          | Purpose                                           | Syntax       |
| ---------------- | ------------------------------------------------- | ------------ |
| **Dot notation** | Organizational hierarchy (which epic owns a task) | `epic-id.N`  |
| **Dependencies** | Execution order (what blocks what)                | `bd dep add` |

These serve different purposes and should be used together.

## Dot Notation: Organizational Hierarchy

Use dot notation to indicate a task belongs to an epic. The format is `parent-id.N` where N is a sequential number.

```bash
# First, create the parent epic
bd create --title="Release 0.7.0" --type=epic --priority=2
# Output: Created nacre-xyz

# Then create child tasks using the epic ID as prefix
bd create --id="nacre-xyz.1" --title="Implement authentication" --type=task
bd create --id="nacre-xyz.2" --title="Write auth tests" --type=task
bd create --id="nacre-xyz.3" --title="Update security docs" --type=task
```

**Benefits:**

- Tasks appear nested under their epic in tree views
- Progress tracking rolls up to the parent epic
- Clear ownership and scope

**Rules:**

- Always use sequential numbering: `.1`, `.2`, `.3`
- The parent ID must exist before creating children
- Children inherit context from parent epic
- Always use description field
- Try to use labels (backend, frontend, docs, infra, etc.)
- When breaking a task into sub-tasks use these:
  - creating a clarifying sub-task (when necessary)
  - creating a pull request
  - reviewing the pull request
- Make these items depend on each other

## Dependencies: Execution Order

Use dependencies to define what must complete before another task can start. This controls `bd ready` output and blocking status.

```bash
# Task 2 depends on Task 1 (Task 1 must finish first)
bd dep add nacre-xyz.2 nacre-xyz.1

# Task 3 also depends on Task 1
bd dep add nacre-xyz.3 nacre-xyz.1

# Task 4 depends on both Task 2 and Task 3
bd dep add nacre-xyz.4 nacre-xyz.2
bd dep add nacre-xyz.4 nacre-xyz.3
```

**Syntax:** `bd dep add <blocked-task> <blocking-task>`

- The first argument is the task that WAITS
- The second argument is the task that BLOCKS

**Benefits:**

- `bd ready` only shows unblocked tasks
- `bd blocked` shows what's waiting and why
- Prevents starting work out of order

## Complete Workflow Example

```bash
# 1. Create epic for the release
bd create --title="User Authentication Feature" --type=epic --priority=1
# → nacre-abc

# 2. Create child tasks (organizational structure)
bd create --id="nacre-abc.1" --title="Implement login endpoint" --type=task
bd create --id="nacre-abc.2" --title="Implement logout endpoint" --type=task

# 3. Create child sub-tasks

bd create --id="nacre-abc.1.1" --title="Investigate login endpoint" --type=task
bd create --id="nacre-abc.1.2" --title="Create login endpoint PR" --type=task
bd create --id="nacre-abc.1.3" --title="Review login endpoint PR" --type=task

# 4. Define execution order (dependencies)
# logout tasks depend on login
bd dep add nacre-abc.2 nacre-abc.1
# review task depends on creation
bd dep add nacre-abc.1.2 nacre-abc.1.1
```

## When to Use Each

| Scenario                            | Use                          |
| ----------------------------------- | ---------------------------- |
| "This task is part of Release X"    | Dot notation: `release-id.N` |
| "This task must wait for that task" | Dependency: `bd dep add`     |
| "Group related work together"       | Dot notation                 |
| "Control work sequence"             | Dependency                   |
| "Track epic progress"               | Dot notation                 |
| "Find what's ready to work on"      | Dependencies via `bd ready`  |

## Summary

1. **Create epic first** - establishes the parent for organizational grouping
2. **Create children with dot notation** - `epic-id.1`, `epic-id.2`, etc.
3. **Add dependencies for execution order** - `bd dep add waiting-task blocking-task`
4. **Use `bd ready`** - find unblocked tasks ready for work
5. **Use `bd blocked`** - see what's waiting and why

## Example output (without subtasks)

```bash
-> bd ready

📋 Ready work (9 issues with no blockers):

1. [P1] [epic] nacre-90b: Release 0.6.0 - Unified Tasks View & UI Refinements
2. [P2] [task] nacre-90b.3: Refine Task Detail header spacing
3. [P2] [feature] nacre-90b.7: Add sorting controls to Child Tasks list
4. [P2] [task] nacre-90b.8: Make search box conditional by page
5. [P2] [task] nacre-90b.10: Refactor and compress CSS
6. [P2] [task] nacre-90b.11: Implement OKLCH color system
7. [P2] [task] nacre-90b.14: Implement Nacre Color System (Fire + Cool Slate)
8. [P3] [task] nacre-90b.12: Document Design System in README
9. [P3] [feature] nacre-90b.13: Implement /palette endpoint for design system preview
```
