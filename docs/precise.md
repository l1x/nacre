# Precise Context Specification

Purpose: Curate precise context for AI coding agents - providing exactly what they need to understand and complete a task, without overwhelming them with irrelevant code.

## Context Components

A well-structured AI prompt consists of:

1.  Instructions - What the agent should do
1.  File Tree - Project structure for navigation context
1.  Codemaps - Structural summaries (signatures, types, interfaces) without implementation (~10x fewer tokens than full files)
1.  Selected Files - Full content of files being modified
1.  Slices - Specific line ranges from large files (e.g., L45-120)

File Selection Modes

| Mode    | Content               | Use Case                            |
| ------- | --------------------- | ----------------------------------- |
| Full    | Complete file         | Files being actively edited         |
| Slices  | Line ranges only      | Large files where only part matters |
| Codemap | Signatures/types only | Reference files, APIs, dependencies |

Key Principles

- Codemaps extract function signatures, class definitions, type declarations - giving structural understanding without implementation bloat
- Slices can turn a 2000-line file into a 50-line extract containing just the relevant function
- Token budgeting matters - leave room for the agent's response (10-20k for plans, more for code generation)

For XML Ticket Generation

The XML should specify:

- Task instructions
- Files needed (with mode: full, codemap, or slice with line ranges)
- Relevant dependencies to include as codemaps
- Any git diff context if continuing work
- Behavioral guidance (role, constraints)
