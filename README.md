# Riptide Rust Proxy MVP

A high-performance upstream proxy implementation in Rust with io_uring support for zero-copy networking.

## Features

✅ **Upstream Connections with Parameter Mapping**
- Weighted provider selection
- Credential parameter mapping (country, city, state, session, time)
- Support for both HTTP and SOCKS5 upstream providers

✅ **User Limits Enforcement**
- **Thread Limit**: Maximum concurrent connections per user
- **Speed Limit**: Throughput limiting with token bucket algorithm
- **Bandwidth Limit**: Total data transfer quota per user

✅ **Zero-Copy Networking**
- io_uring support on Linux for minimal syscalls
- Automatic fallback to buffered copy on unsupported systems
- Splice-based zero-copy data transfer

✅ **Protocol Support**
- HTTP CONNECT tunneling
- SOCKS5 proxy (with username/password auth)
- Dual authentication on all ports

## Building

```bash
cd rust

# Build release version with io_uring support (Linux only)
cargo build --release

# Build without io_uring (portable)
cargo build --release --no-default-features
```

## Configuration

Create a `config.json` file (see example in `../cmd/riptide/config.json`):

```json
{
  "server": {
    "logging": "info",
    "inflation": 0,
    "sni-check": false,
    "node-name": "rust-node",
    "update-interval": 10,
    "source-ips": [],
    "retries": {
      "max-retries": 3,
      "timeout": 5
    }
  },
  "api": [
    {
      "name": "control-plane",
      "base-url": "http://127.0.0.1:8000/api",
      "api-key": "your-api-key",
      "default-package": "residential",
      "legacy": false,
      "ports": {
        "userpass": "8080-8080"
      }
    }
  ],
  "upstream": [
    {
      "name": "residential",
      "ips": ["http://upstream.example.com:10000"],
      "user": "upstream-user",
      "password": "upstream-pass",
      "mapping": {
        "country": "country",
        "city": "city",
        "state": "state",
        "session": "session",
        "time": "time"
      },
      "separator": "-",
      "format-in": "username",
      "weight": 100,
      "package": "residential"
    }
  ]
}
```

## Running

```bash
# Start the proxy
./target/release/riptide --config config.json

# Or with cargo
cargo run --release -- --config config.json
```

## Testing

The MVP includes a comprehensive test client:

```bash
# Build test client
cargo build --release --bin test-client

# Single connection test
./target/release/test-client \
  --proxy 127.0.0.1:8080 \
  --username testuser \
  --password testpass \
  --target google.com:80 \
  --connections 1 \
  --duration 10

# Load test with 100 concurrent connections
./target/release/test-client \
  --proxy 127.0.0.1:8080 \
  --username testuser \
  --password testpass \
  --target google.com:80 \
  --connections 100 \
  --duration 30

# Test with SOCKS5
./target/release/test-client \
  --proxy 127.0.0.1:8080 \
  --username testuser \
  --password testpass \
  --target google.com:80 \
  --socks5 \
  --connections 50 \
  --duration 20
```

### Test Client Options

- `--proxy, -p`: Proxy address (default: 127.0.0.1:8080)
- `--username, -u`: Authentication username
- `--password, -P`: Authentication password
- `--target, -t`: Target host:port (default: google.com:80)
- `--connections, -c`: Number of concurrent connections (default: 1)
- `--duration, -d`: Test duration in seconds (default: 10)
- `--socks5, -s`: Use SOCKS5 instead of HTTP CONNECT

The test client will report:
- ✅ Successful connections and bytes transferred
- ❌ Failed connections with error messages
- 📊 Final statistics: total time, data transferred, throughput

## Parameter Mapping

The proxy supports credential parameter mapping to route connections through specific upstream providers with custom attributes:

### Usage

Format your username with dash-separated key-value pairs:

```
username-country-us-city-newyork-session-abc123
```

### Supported Parameters

- `country`: Target country code (e.g., `us`, `uk`, `de`)
- `state`: Target state/region
- `city`: Target city
- `session`: Session identifier for sticky sessions
- `time`: Session duration/TTL

### Example

```bash
curl -x http://user-country-us-session-test123:password@127.0.0.1:8080 https://httpbin.org/ip
```

The upstream credential will be built as:
```
upstream-user-country-us-session-test123
```

## Monitoring

The proxy logs structured JSON events:

- 🚀 Startup and initialization
- 🔄 User sync from control plane
- 📊 Statistics collection
- 📤 Reporting to control plane
- 🎯 Provider selection
- 📡 Connection handling
- ⚠️ Errors and warnings

## Performance

### Auto-Scaling

The proxy auto-scales based on installed RAM:
- Buffer sizes: 8KB fixed
- Channel sizes: RAM_GB × 10000 (capped at 200K)
- Expected throughput: ~1K req/sec per GB RAM

### io_uring Benefits (Linux only)

- **Zero-copy**: Data moves directly between sockets via kernel pipes
- **Minimal syscalls**: Batch operations reduce context switching
- **Lower CPU**: No user-space buffer copying for tunnel data
- **Better latency**: Reduced overhead per connection

### Fallback Behavior

On systems without io_uring support (non-Linux, old kernels):
- Automatic fallback to buffered copy
- 8KB buffers per direction
- Still efficient, just not zero-copy

## Architecture

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ HTTP/SOCKS5
       │
┌──────▼──────────────────────────────┐
│  Riptide Proxy (Port 8080)          │
│  ┌────────────────────────────────┐ │
│  │  Protocol Handler               │ │
│  │  - HTTP CONNECT                 │ │
│  │  - SOCKS5 TCP CONNECT           │ │
│  │  - Authentication               │ │
│  └─────────────┬───────────────────┘ │
│                │                      │
│  ┌─────────────▼───────────────────┐ │
│  │  Limits Enforcement             │ │
│  │  - Thread cap check             │ │
│  │  - Connection rate limit        │ │
│  │  - Bandwidth quota check        │ │
│  │  - Throughput rate limiting     │ │
│  └─────────────┬───────────────────┘ │
│                │                      │
│  ┌─────────────▼───────────────────┐ │
│  │  Upstream Selector              │ │
│  │  - Weighted provider selection  │ │
│  │  - Parameter mapping            │ │
│  │  - Credential building          │ │
│  └─────────────┬───────────────────┘ │
│                │                      │
│  ┌─────────────▼───────────────────┐ │
│  │  Tunnel (io_uring / buffered)   │ │
│  │  - Zero-copy splice (Linux)     │ │
│  │  - Buffered copy (fallback)     │ │
│  │  - Statistics tracking          │ │
│  └─────────────┬───────────────────┘ │
└────────────────┼─────────────────────┘
                 │
       ┌─────────▼─────────┐
       │  Upstream Provider │
       │  (HTTP or SOCKS5)  │
       └─────────┬──────────┘
                 │
       ┌─────────▼──────────┐
       │   Target Website   │
       └────────────────────┘

Background Tasks:
┌────────────────────────────┐
│  User Sync (every N sec)   │ ───▶ GET /riptide
│  - Fetch users & plans     │      GET /blacklists
│  - Update limits           │
│  - Refresh credentials     │
└────────────────────────────┘

┌────────────────────────────┐
│  Reporter (every N sec)    │ ───▶ POST /riptide/report
│  - Collect statistics      │      {key, traffic, threads, throughput}
│  - Calculate throughput    │
│  - Send to control plane   │
└────────────────────────────┘
```

## MVP Scope

This MVP implements core functionality:

✅ Upstream connection with parameter mapping
✅ User limits (threads, speed, bandwidth)
✅ io_uring zero-copy support (Linux)
✅ HTTP CONNECT and SOCKS5 protocols
✅ User sync from control plane API
✅ Statistics reporting
✅ Test client for validation

Not included in MVP (per PRD, for future iterations):
- IPv6 / ISP local tunnel logic
- Domain/port blacklists
- TLS SNI preflight
- Sticky sessions with TTL
- ClickHouse integration
- IP authentication (ipauth ports)
- UDP ASSOCIATE for SOCKS5
- Full HTTP proxy (non-CONNECT)

## Control Plane API

### GET /riptide?node={node-name}

Returns users and plans:

```json
[
  {
    "username": "user1",
    "password": "pass1",
    "user_id": 123,
    "user_type": "residential",
    "plan": {
      "id": 456,
      "status": "active",
      "max_threads": 100,
      "max_throughput": 50,
      "max_bytes": 10737418240,
      "bytes_used": 1073741824
    }
  }
]
```

### POST /riptide/report

Sends statistics:

```json
[
  {
    "key": "residential:user1:456:123",
    "traffic": 524288000,
    "current_threads": 5,
    "current_throughput": 52.43
  }
]
```

## License

Proprietary - Internal use only

