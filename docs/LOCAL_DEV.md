# Local Development Fix

## Problem

Running `cargo run -p ghostwire-server` failed with:

```
Runtime received malformed or incorrect args, missing required argument --port
```

This happened because the `main.rs` uses Shuttle's runtime (`#[shuttle_runtime::main]`), which expects specific command-line arguments that are only provided when running through `cargo shuttle run`.

## Solution

Created a **separate binary** for local development that doesn't use Shuttle:

### Files Changed

1. **[`server/Cargo.toml`](server/Cargo.toml)**

   - Added two binary targets:
     - `ghostwire-server` - For Shuttle deployment (uses `src/main.rs`)
     - `ghostwire-local` - For local development (uses `src/local.rs`)

2. **[`server/src/local.rs`](server/src/local.rs)** (NEW)

   - Standalone binary with `#[tokio::main]`
   - No Shuttle dependencies
   - Listens on `0.0.0.0:8080`
   - Same functionality as Shuttle version

3. **[`server/src/main.rs`](server/src/main.rs)**
   - Removed unused `local_main()` function
   - Kept only Shuttle-specific code

## Usage

### Local Development (NEW)

```bash
cargo run --bin ghostwire-local
```

Output:

```
INFO ghostwire_server: 🚀 Starting GhostWire Relay Server (Local Mode)
INFO ghostwire_server: 👻 GhostWire Relay listening on http://0.0.0.0:8080
INFO ghostwire_server: 📡 WebSocket endpoint: ws://0.0.0.0:8080/ws
```

### Shuttle Deployment (Unchanged)

```bash
cd server
cargo shuttle deploy
```

## Why Two Binaries?

| Binary             | Purpose            | Runtime                 | Entry Point    |
| ------------------ | ------------------ | ----------------------- | -------------- |
| `ghostwire-server` | Shuttle deployment | `shuttle_runtime::main` | `src/main.rs`  |
| `ghostwire-local`  | Local development  | `tokio::main`           | `src/local.rs` |

**Benefits:**

- ✅ No need to install `cargo-shuttle` for local testing
- ✅ Faster compile times (no Shuttle dependencies)
- ✅ Same functionality, different entry points
- ✅ Clean separation of concerns

## Testing

### Start Server

```bash
cargo run --bin ghostwire-local
```

### Connect Client

```bash
cargo run -p ghostwire-client alice ws://localhost:8080/ws
```

## Documentation Updated

- ✅ [`QUICKSTART.md`](../QUICKSTART.md)
- ✅ [`docs/SERVER.md`](SERVER.md)

All references to `cargo run -p ghostwire-server` have been updated to `cargo run --bin ghostwire-local`.
