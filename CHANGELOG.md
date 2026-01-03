# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.0] - 2026-01-03

### Added

- **Graph View**: Horizontal org-chart tree visualization replacing Cytoscape.js
- **Graph View**: Horizontal epic selector with pagination arrows
- **Graph View**: Curved SVG Bezier connectors with gradient fade
- **Graph View**: Status and type indicator dots on nodes
- **Graph View**: Hover glow effect on child nodes
- **PRDs View**: Horizontal PRD selector dropdown (matching graph style)
- **Theming**: Theme switcher with 6 themes (nacre-dark, nacre-light, catppuccin-mocha/macchiato/frappe/latte)
- **Theming**: Semantic color palette with theme-aware chart colors
- **Header**: Project folder name display below logo
- **Metrics**: Refactored to pure functions for improved testability (17 new unit tests)

### Changed

- **Graph View**: Tasks display in multiple rows with wrapping
- **Graph View**: Legend width matches epic selector box
- **Board View**: Added max-width and centering for consistency
- **Charts**: Use yellow instead of grey for Resolved bars

### Fixed

- **Metrics**: Cycle time calculation now correctly excludes issues without InProgress transitions
- **Metrics**: EventType parsing includes Deleted event type
- **Metrics**: Always show cycle time chart even with limited data
- **Board View**: Theme selector now visible
- **Markdown**: Table borders display correctly
- **Graph View**: Lighter nodes and darker lines for better readability

## [0.8.0] - 2026-01-01

### Added

- **Markdown**: GFM tables and strikethrough support via `pulldown_cmark` options
- **Markdown**: XML syntax highlighting for code blocks
- **Metrics**: Chart navigation bar on `/metrics` page with anchor links
- **Logging**: Human-readable latency formatting (µs/ms/s based on magnitude)
- **Dev Tools**: ESLint configuration with security plugin for TypeScript
- **Dev Tools**: TypeScript lint tasks in mise (`lint-ts`, `lint-all`)

### Changed

- **Static Assets**: Refactored to use `include_dir` crate for cleaner code
  - Single `include_dir!` macro replaces 5 separate `include_str!` calls
  - Reduced static asset handling code from ~80 lines to ~30 lines
  - Added generic `serve_static()` handler for future extensibility

### Fixed

- **Frontend**: Object injection warnings in `navigation.ts` (use `.at()` instead of bracket notation)
- **Frontend**: Type guard for sort key validation in `sorting.ts`
- **Frontend**: Object injection warnings in `board.ts`
- **Frontend**: Remove unused `err` variable in `toast.ts`
- **UI**: Chart nav alignment with metrics grid (CSS grid layout)
- **UI**: Scroll margin for anchor navigation

## [0.7.0] - 2025-12-30

Initial tracked release.
