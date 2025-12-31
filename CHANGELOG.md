# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
