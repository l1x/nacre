# Nacre

A local-first web interface for [Beads](https://github.com/beads-project/beads) issue tracking.

## Features

- **Issues List** - View all issues grouped by epic, sorted by status priority
- **Board View** - Kanban-style board with drag-and-drop status updates
- **Epics View** - Track epic progress with completion percentages
- **PRDs View** - Browse and read product requirement documents (Markdown)
- **Issue Details** - View full issue information including description, acceptance criteria, and metadata

## Installation

### Prerequisites

- Rust 1.75+ (uses Rust 2024 edition)
- [Beads CLI](https://github.com/beads-project/beads) installed and configured

### Build from source

```bash
git clone https://github.com/l1x/nacre.git
cd nacre
cargo build --release
```

The binary will be at `target/release/nacre`.

## Usage

```bash
# Start the server (default: http://127.0.0.1:3000)
nacre

# Specify host and port
nacre --host 0.0.0.0 --port 8080

# Open browser automatically
nacre --open

# Custom static files directory
nacre --static-dir ./frontend/public
```

### Command-line Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--host` | | `127.0.0.1` | Host to bind to |
| `--port` | `-p` | `3000` | Port to listen on |
| `--open` | `-o` | `false` | Open browser automatically |
| `--static-dir` | `-s` | `frontend/public` | Directory for static files |

## Development

### Project Structure

```
nacre/
├── src/
│   ├── main.rs      # Web server and routes
│   └── beads.rs     # Beads CLI integration
├── templates/       # Askama HTML templates
├── frontend/
│   └── public/      # Static assets (CSS, JS)
└── docs/
    └── prds/        # Product requirement documents
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

- **0.2.0** (2024-12-25) - Current release
- **0.1.0** - Initial release

## License

MIT
