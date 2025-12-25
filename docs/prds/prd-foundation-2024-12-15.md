# PRD: Nacre

## 1. Overview
**Nacre** is a local-first web interface for the [Beads](https://github.com/steveyegge/beads) issue tracking system. While Beads provides a powerful distributed, git-backed graph database for AI agents, interacting with it solely via CLI can be cumbersome for human users. Nacre acts as the polished shell around this data, providing a visual, interactive layer that allows developers to collaborate seamlessly with their AI coding agents. It offers real-time visualization of tasks, Kanban boards for workflow management, and keyboard-centric navigation for efficiency.

## 2. Goals
1.  **Visual Management**: Provide a graphical interface (Kanban, Lists) to visualize the state of the Beads database.
2.  **Real-Time Collaboration**: Ensure the UI reflects changes made by AI agents or background processes instantly without manual refresh.
3.  **Efficiency**: Enable "Zero Setup" installation and comprehensive keyboard navigation to match the speed of CLI workflows.
4.  **Seamless Integration**: Operate directly on the local Beads database (`.beads` directory) and utilize the `bd` binary where necessary.

## 3. User Stories
1.  **As a Developer**, I want to view my issues on a Kanban board, so that I can quickly assess the status of tasks (Blocked, Ready, In Progress, Closed).
2.  **As a Developer**, I want to edit issue details (title, status) inline, so that I can make quick updates without context switching to a text editor.
3.  **As a Developer**, I want to see the progress of Epics, so that I can track high-level goals and their sub-tasks.
4.  **As a Developer**, I want the UI to update automatically when my AI agent creates or modifies a task, so that I am always looking at the current state.
5.  **As a Developer**, I want to navigate the interface using only my keyboard, so that I can maintain my coding flow.
6.  **As a User**, I want to start the application with a single command from my project root (e.g., `nacre start`), so that I don't have to configure complex servers.

## 4. Functional Requirements

### 4.1. Core Application
1.  **Server Start**: The application must start a local web server and optionally open the browser (`--open`).
2.  **CLI Arguments**:
    - Support `--host` and `--port` to override defaults.
    - Support environment variables `BD_BIN`, `NACRE_RUNTIME_DIR`.
3.  **Data Source**: Connect to the local Beads database (typically `.beads` directory) and/or interact via the `bd` CLI.

### 4.2. Views & Navigation
1.  **Navigation Menu**: Persistent navigation to switch between Issues, Epics, and Board views.
2.  **Issues View**:
    - List all issues.
    - **Search & Filter**: Ability to filter issues by text or status.
    - **Inline Editing**: Edit basic issue fields directly in the list.
3.  **Epics View**:
    - Display a hierarchical list of epics.
    - **Progress Indicators**: Visual progress bar showing completion of child tasks.
    - **Expansion**: Expand/collapse epics to view child issues.
4.  **Board View**:
    - Kanban layout with columns: **Blocked**, **Ready**, **In Progress**, **Closed**.
    - Drag-and-drop (optional) or state-change mechanisms to move cards.

### 4.3. Real-Time Updates
1.  **Database Monitoring**: The backend must watch the Beads database/files for changes.
2.  **Push Updates**: Push changes to the frontend (via WebSockets or SSE) to update the UI immediately upon external modification.

### 4.4. Keyboard Shortcuts
1.  **Navigation**: Use arrow keys or `j`/`k` to navigate lists and boards.
2.  **Actions**: dedicated keys to open details, edit, or change status without mouse interaction.

## 5. Non-functional Requirements

1.  **Performance**:
    - Application startup time should be under 2 seconds.
    - UI latency for local actions should be <100ms.
2.  **Compatibility**:
    - Fully supported on macOS and Linux.
    - Functional on Windows (with known limitations regarding process signals).
3.  **Dependency**:
    - Must require the `bd` binary to be present or accessible.
4.  **Tech Stack**:
    - Language: Rust for performance and reliability.
    - Web Framework: Axum for async HTTP server and routing.
    - Templating: Askama for compile-time checked HTML templates.
    - CLI: argh for argument parsing.
    - Frontend: Vanilla JavaScript with minimal dependencies.

## 6. Non-Goals
1.  **Remote Hosting**: This is a local-first tool; hosting as a multi-user SaaS is not a goal.
2.  **Authentication**: No user login required as it runs locally.
3.  **Complex Graph Visualization**: While Beads is a graph, this UI focuses on Lists and Kanban; complex node-graph rendering is secondary to workflow management.

## 7. Success Metrics
1.  **Adoption**: GitHub stars, forks, and community contributions.
2.  **Engagement**: Frequency of use (anecdotal feedback from developers using it with their AI agents).
3.  **Efficiency**: Reduction in time to triage issues compared to using `bd` CLI list commands (measured via user feedback).

## 8. Design Considerations
-   **Minimalist UI**: Focus on data density and readability (Nacre/Shell aesthetic).
-   **Dark Mode**: Preferred by the target audience (developers).
-   **Tech Constraints**: Must handle concurrent file access if the AI agent is writing to `.beads` simultaneously.

## 9. Open Questions
1.  What is the specific protocol for "push-only" updates mentioned in the docs? (Assumed SSE/WebSocket).
2.  Does the UI need to support all `bd` commands (e.g., merge, branch) or just issue management? (Assumed Issue Management primarily).
