# IronFix Examples

This directory contains example implementations of FIX servers and clients for each supported FIX version.

## Structure

```
examples/
├── fix40/          # FIX 4.0 examples
│   ├── client.rs
│   └── server.rs
├── fix41/          # FIX 4.1 examples
│   ├── client.rs
│   └── server.rs
├── fix42/          # FIX 4.2 examples
│   ├── client.rs
│   └── server.rs
├── fix43/          # FIX 4.3 examples
│   ├── client.rs
│   └── server.rs
├── fix44/          # FIX 4.4 examples
│   ├── client.rs
│   └── server.rs
├── fix50/          # FIX 5.0 examples (FIXT.1.1 transport)
│   ├── client.rs
│   └── server.rs
├── fix50sp1/       # FIX 5.0 SP1 examples
│   ├── client.rs
│   └── server.rs
├── fix50sp2/       # FIX 5.0 SP2 examples
│   ├── client.rs
│   └── server.rs
└── common/         # Shared utilities
    └── mod.rs
```

## Running Examples

### Start a server (e.g., FIX 4.4):

```bash
cargo run --example fix44_server
```

### Start a client (e.g., FIX 4.4):

```bash
cargo run --example fix44_client
```

## Configuration

By default, examples use:
- **Host**: `127.0.0.1`
- **Port**: `9876` (varies by version)
- **SenderCompID**: `CLIENT` / `SERVER`
- **TargetCompID**: `SERVER` / `CLIENT`

Environment variables can override defaults:
- `FIX_HOST` - Server hostname
- `FIX_PORT` - Server port
- `FIX_SENDER` - SenderCompID
- `FIX_TARGET` - TargetCompID
