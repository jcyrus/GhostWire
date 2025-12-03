# Changelog

All notable changes to GhostWire will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-12-03

### Added

- **Multi-Channel System**: Support for global chat and direct messages (DMs)
  - Global channel (`# global`) for public conversations
  - Direct message channels (`@ username`) for private 1-on-1 chats
  - Auto-creation of DM channels when receiving messages
  - Channel switching with keyboard shortcuts (`h/l` + `Tab`)
  - Unread message count badges on channels
- **Enhanced Telemetry Panel**: Real-time statistics and monitoring
  - Dynamic network activity chart (last 60 seconds)
  - Connection uptime tracking
  - Latency gauge with color-coded status
  - Message throughput statistics
  - Active channel display
  - User and channel count
  - Server time display (UTC)
- **User Discovery**: Automatic user roster population
  - Users appear when they send messages
  - Online/offline status indicators
  - Last seen timestamps for offline users
  - User selection for DM creation (`J/K` keys)
- **Client Architecture**: Async/sync split design
  - Main thread handles UI rendering (60fps target)
  - Separate async task for WebSocket communication
  - `mpsc` channels for thread-safe message passing
- **Server Implementation**: Dumb relay pattern
  - WebSocket-based message broadcasting
  - Connection management and client tracking
  - Shuttle.rs deployment support
  - Local development mode
- **TUI Features**: Ratatui-based interface
  - Three-panel layout (channels, chat, telemetry)
  - Vim-style keyboard navigation
  - Message timestamps
  - System message support
  - Scrollable chat history
  - Input mode with cursor support

### Technical Details

- **Protocol**: JSON-based wire format with channel routing
- **Dependencies**: Tokio for async runtime, Ratatui for TUI, Axum for server
- **Workspace**: Monorepo structure with client and server packages
- **Build**: Rust 2021 edition, clippy-clean with strict warnings

### Documentation

- Comprehensive README with ASCII art logo
- Client architecture documentation (`CLIENT.md`)
- Server deployment guide (`SERVER.md`)
- Quick start guide (`QUICKSTART.md`)
- Channel system user guide (`CHANNELS.md`)
- Feature implementation details (`FEATURES.md`)

### Known Limitations

- No encryption (messages are plain JSON)
- No message persistence (ephemeral chat)
- No group channels yet (reserved for future)
- Server broadcasts all messages to all clients (no server-side filtering)

[Unreleased]: https://github.com/jcyrus/GhostWire/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jcyrus/GhostWire/releases/tag/v0.1.0
