# PRD: Foundation v2

## Context

**Nacre** is a agentic project management tool with a web interface using the [Beads](https://github.com/steveyegge/beads) issue tracking system. While Beads provides a powerful git-backed database for coding agents, interacting with it solely via CLI can be cumbersome for human users. Nacre is a web interface is an interactive layer that allows developers to collaborate seamlessly with their AI coding agents. It offers Kanban boards for workflow management and some keyboard navigation for efficiency.

## Goals

1.  **Visual Management**: Provide a graphical interface (Kanban) to visualize the state of the tasks in Beads.
1.  **Efficiency**: Enable single binary deployments and zero-config
1.  **Seamless Integration**: Operate directly on the local Beads database (`.beads` directory) and utilize the `bd` binary where necessary.

## Functional Requirements

### Core Application

1.  **Server**: The application must start a local web server and optionally open the browser (`--open`).
1.  **CLI Arguments**: Support `--host` and `--port` to override defaults.
1.  **Data Source**: Connect to the local Beads database using the `bd` CLI.

### Views & Navigation

1.  **Navigation Menu**: Persistent navigation to switch between Dashboard, Tasks, Metrics, PRDs and Board views.
1.  **Search**: Ability to filter issues by text or status.
1.  **Dashboard View**:
    - Ticket activity overview
1.  **Tasks View**:
    - Viewing all or individual tasks
    - Display a hierarchical structure when possible
1.  **Board View**:
    - Standard simple Kanban like lanes
    - Drag-and-drop to move cards.
1.  **Metrics View**:
    - Simple development efficiency numbers and charts
1.  **PRDs view**:
    - Viewing product requirements docs

### Keyboard Shortcuts

1.  **Navigation**: Use arrow keys or `j`/`k` to navigate on board view.
1.  **Actions**: dedicated keys to open details, or go back.

## Non-functional Requirements

1.  **Performance**:
    - Application startup time should be under 256 milliseconds.
    - UI latency for local actions should be <256ms.
1.  **Compatibility**:
    - Fully supported on macOS and Linux.
    - Functional on Windows (with known limitations regarding process signals).
1.  **Dependency**:
    - Must require the `bd` binary to be present or accessible.
1.  **Tech Stack**:
    - Language: Rust for performance and reliability.
    - Web Framework: Axum for async HTTP server and routing.
    - Templating: Askama for compile-time checked HTML templates.
    - CLI: argh for argument parsing.
    - Frontend: Vanilla TypeScript without dependencies.
    - mise for managing tasks used to build and test
    - bun for building app.js

## Non-Goals

1.  **Remote Hosting**: This is a local-first tool; hosting as a multi-user SaaS is not a goal.
1.  **Authentication**: No user login required as it runs locally.

## Success Metrics

1. 100% of the user workflows are covered
