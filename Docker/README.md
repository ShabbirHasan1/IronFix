# IronFix Docker Images

Dockerfiles for building and running IronFix FIX protocol servers.

## Available Servers

| Dockerfile | FIX Version | Default Port |
|------------|-------------|--------------|
| `fix40.Dockerfile` | FIX 4.0 | 9870 |
| `fix41.Dockerfile` | FIX 4.1 | 9871 |
| `fix42.Dockerfile` | FIX 4.2 | 9872 |
| `fix43.Dockerfile` | FIX 4.3 | 9873 |
| `fix44.Dockerfile` | FIX 4.4 | 9876 |
| `fix50.Dockerfile` | FIX 5.0 (FIXT.1.1) | 9880 |
| `fix50sp1.Dockerfile` | FIX 5.0 SP1 | 9881 |
| `fix50sp2.Dockerfile` | FIX 5.0 SP2 | 9882 |
| `fast.Dockerfile` | FAST Protocol | 9890 |

## Building

Build from the repository root:

```bash
# Build FIX 4.4 server
docker build -f Docker/fix44.Dockerfile -t ironfix-fix44:latest .

# Build all servers
for v in 40 41 42 43 44 50 50sp1 50sp2; do
  docker build -f Docker/fix${v}.Dockerfile -t ironfix-fix${v}:latest .
done
```

## Running

```bash
# Run FIX 4.4 server
docker run -d -p 9876:9876 --name fix44-server ironfix-fix44:latest

# Run with custom configuration
docker run -d -p 9876:9876 \
  -e FIX_HOST=0.0.0.0 \
  -e FIX_PORT=9876 \
  -e FIX_SENDER=MY_SERVER \
  -e FIX_TARGET=MY_CLIENT \
  --name fix44-server ironfix-fix44:latest
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `FIX_HOST` | Bind address | `0.0.0.0` |
| `FIX_PORT` | Listen port | Version-specific |
| `FIX_SENDER` | SenderCompID | `SERVER` |
| `FIX_TARGET` | TargetCompID | `CLIENT` |

## Image Details

- **Build stage**: `rust:1.92.0-alpine3.23`
- **Runtime stage**: `alpine:3.23`
- **Binary**: Statically linked with musl libc
