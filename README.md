# Nacre

A local-first web interface for [Beads](https://github.com/beads-project/beads) issue tracking.

## Features

- **Dashboard** - Project overview with stats, epic progress, blocked and in-progress issues
- **Board View** - Kanban-style board with drag-and-drop status updates and column visibility toggle
- **Epics View** - Track epic progress with completion percentages and expandable child issues
- **Issues List** - Hierarchical tree view with type filtering and expand/collapse controls
- **Metrics View** - Lead time, cycle time, throughput, and ticket activity charts
- **PRDs View** - Browse and read product requirement documents (Markdown)
- **Issue Details** - View and edit issues with description, acceptance criteria, labels, and timestamps

## Installation

### Prerequisites

- Rust 1.75+ (uses Rust 2024 edition)
- [Beads CLI](https://github.com/beads-project/beads) installed and configured

### Build from source

```bash
git clone https://github.com/l1x/nacre.git
cd nacre
cargo build --release
# or using mise
mise build-prod
```

The binary will be at `target/release/nacre`.

### Binary Info

- **Size**: 3.8 MB (release build)
- **Platform**: macOS (Darwin)

**Linked Libraries** (macOS):

```
CoreText.framework
CoreGraphics.framework
libiconv.2.dylib
CoreFoundation.framework
libSystem.B.dylib
```

## Usage

```bash
# Open browser automatically
nacre --open
```

### Command-line Options

| Option         | Short | Default           | Description                |
| -------------- | ----- | ----------------- | -------------------------- |
| `--host`       |       | `127.0.0.1`       | Host to bind to            |
| `--port`       | `-p`  | `3000`            | Port to listen on          |
| `--open`       | `-o`  | `false`           | Open browser automatically |

## Development

### Project Structure

```
nacre/
├── src/
│   ├── main.rs      # Web server and routes
│   └── beads.rs     # Beads CLI integration
├── templates/       # Askama HTML templates
├── frontend/
│   ├── src/         # TypeScript source code
│   │   ├── modules/ # Modular frontend logic
│   │   └── main.ts  # Entry point
│   └── public/      # Static assets (CSS, JS)
└── docs/
    └── prds/        # Product requirement documents
```

### Prerequisites

- **Rust 1.75+**
- **Bun** (for building frontend assets)

### Frontend Development

The frontend logic is written in modular TypeScript (`frontend/src/`) and bundled into a single file (`frontend/public/app.js`) which is then embedded into the Rust binary.

To build the frontend:

```bash
# Build TypeScript to app.js
mise run build-js-prod

# Build TypeScript to test.js (for verification)
mise run build-js-test
```

### Running in development

```bash
# Run with debug logging
RUST_LOG=nacre=debug cargo run

# Run tests
cargo test

# Run linter
cargo clippy
```

### Tech Stack

- **[Axum](https://github.com/tokio-rs/axum)** - Web framework
- **[Askama](https://github.com/djc/askama)** - Type-safe HTML templates
- **[Tokio](https://tokio.rs/)** - Async runtime
- **[pulldown-cmark](https://github.com/raphlinus/pulldown-cmark)** - Markdown rendering

## Version History

### 0.7.0 (2025-12-30)

**New Features**
- Syntax highlighting for code blocks in PRDs and task descriptions
- Dynamic light/dark theme switching for code blocks (follows theme toggle)
- Dark theme: Monokai Pro
- Light theme: Ayu Light

**Supported Languages**
- Rust, Python, JavaScript, TypeScript
- JSON, HTML, CSS
- Bash/Shell, TOML, YAML

**Dependencies**
- Added `autumnus` crate for tree-sitter based syntax highlighting
- Added `pulldown-cmark` for markdown parsing with code block detection

### 0.6.0 (2025-12-29)

**Security**
- Fixed path traversal vulnerability in PRDs handler
- Sanitized error messages to prevent information disclosure

**New Features**
- Added Activity Heat Map chart (hours on X-axis, days on Y-axis)
- Implemented TypeScript error handling with toast notifications

**Bug Fixes**
- Fixed ToastManager DOM initialization race condition

**Testing**
- Added comprehensive API endpoint integration tests
- Added cross-feature integration tests
- Split integration tests into multi-file structure for parallel execution

### 0.5.1 (2025-12-28)

**Charts & Dashboard**
- Replaced static SVG charts with pure HTML/CSS implementation for better performance and theming
- Added Ticket Activity chart to Dashboard
- Implemented dynamic axes, arrowheads, and grid lines for all charts
- Fixed dark mode visibility for chart grids
- Removed legacy metrics cards from Dashboard

**Fixes**
- Fixed `Dependency` deserialization (missing `created_at`/`created_by` fields)
- Fixed chart rendering issues (invisible bars, alignment)
- Removed pointer cursors from non-clickable charts

### 0.4.0 (2025-12-27)

**Metrics & Charts**
- Standardized all charts (Ticket Activity, Cycle Time, Throughput, Lead Time) to use consistent date-based stacked bar styling with dark theme
- Added Cycle Time and Throughput charts to Metrics view
- Charts now linkable via anchor tags

**Issue Management**
- Display subtasks in Issue Detail view
- Hide 'Deferred' column by default in Board view

### 0.3.0 (2025-12-26)

**Metrics & Activity**
- Fixed Avg Cycle Time calculation (EventType serde alignment with bd activity JSON)
- Display cycle time in minutes instead of hours for better precision
- Added UTC timestamps to tracing logs

**Board View**
- Colored top borders per column status (Open=orange, In Progress=blue, Blocked=red, Deferred=purple, Closed=gray)
- Matching count badge colors in column headers
- Simplified issue type styling (left border only, removed gradient backgrounds)

**Epics View**
- Reduced epic card height with tighter padding and margins

### 0.2.0 (2025-12-26)

**Template System**
- DRY refactoring with Askama includes (`_head.html`, `_nav.html`, `_header.html`)
- Dynamic page titles and active navigation highlighting

**Frontend**
- Complete TypeScript migration (modular architecture in `frontend/src/modules/`)
- Backspace navigation support
- Global keyboard shortcuts on all views

**Views**
- Hierarchical tree view for Issues page
- Graph view placeholder (dependency visualization coming soon)
- Board card titles now clickable
- Column visibility toggle on Board view
- Redesigned issue detail UI with edit functionality

**Fixes**
- Status enum serialization (snake_case consistency)
- Board drag-and-drop status updates

### 0.1.0 (2025-12-25)

**Core Features**
- Issues list view grouped by epic
- Kanban board with drag-and-drop
- Epics view with progress tracking
- PRDs view with Markdown rendering
- Issue detail view

**Infrastructure**
- Axum web server with Askama templates
- Beads CLI integration
- Request logging with correlation IDs
- Dynamic project name from directory
- Pearl-inspired favicon

## License

MIT
