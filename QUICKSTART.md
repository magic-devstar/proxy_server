# Quick Start Guide - Riptide Rust MVP

## Prerequisites

- Rust toolchain (1.70+): https://rustup.rs/
- Linux (for io_uring support) or any OS (fallback mode)
- Control plane API running (or mock data)

## 1. Build the Proxy

```bash
cd rust

# On Linux (with io_uring)
cargo build --release

# On other systems (without io_uring)
cargo build --release --no-default-features

# Or use the build script
chmod +x build.sh
./build.sh
```

## 2. Configure

Create `config.json` based on `config.example.json`:

```json
{
  "server": {
    "logging": "info",
    "node-name": "test-node",
    "update-interval": 10,
    "source-ips": [],
    "retries": { "max-retries": 3, "timeout": 5 }
  },
  "api": [{
    "name": "test-api",
    "base-url": "http://127.0.0.1:8000/api",
    "api-key": "test-key",
    "default-package": "residential",
    "legacy": false,
    "ports": { "userpass": "8080-8080" }
  }],
  "upstream": [{
    "name": "test-upstream",
    "ips": ["http://your-upstream.com:10000"],
    "user": "upstream-user",
    "password": "upstream-pass",
    "mapping": {
      "country": "country",
      "session": "session"
    },
    "separator": "-",
    "format-in": "username",
    "weight": 100,
    "package": "residential"
  }]
}
```

## 3. Run the Proxy

```bash
./target/release/riptide --config config.json
```

You should see:
```
🚀 Riptide Rust Proxy starting...
✅ Configuration loaded successfully
📋 Config: 1 API endpoints, 1 upstream providers
🌐 Started proxy server on port 8080
✅ All servers started successfully
```

## 4. Test with Test Client

### Single Connection Test

```bash
./target/release/test-client \
  --proxy 127.0.0.1:8080 \
  --username testuser \
  --password testpass \
  --target httpbin.org:80 \
  --connections 1 \
  --duration 5
```

### Load Test (100 concurrent connections)

```bash
./target/release/test-client \
  --proxy 127.0.0.1:8080 \
  --username testuser \
  --password testpass \
  --target httpbin.org:80 \
  --connections 100 \
  --duration 30
```

### SOCKS5 Test

```bash
./target/release/test-client \
  --proxy 127.0.0.1:8080 \
  --username testuser \
  --password testpass \
  --target httpbin.org:80 \
  --socks5 \
  --connections 50 \
  --duration 20
```

## 5. Test with curl

### HTTP CONNECT (HTTPS)

```bash
curl -x http://testuser:testpass@127.0.0.1:8080 https://httpbin.org/ip
```

### With Parameter Mapping

```bash
# Route through US with session ID
curl -x http://testuser-country-us-session-abc123:testpass@127.0.0.1:8080 https://httpbin.org/ip

# Route through specific city
curl -x http://testuser-country-uk-city-london:testpass@127.0.0.1:8080 https://httpbin.org/ip
```

## 6. Verify Limits

### Test Thread Limit

Run more connections than the thread limit:

```bash
# If user has max_threads = 10, try 20 connections
./target/release/test-client -p 127.0.0.1:8080 -u testuser -P testpass -c 20 -d 60
```

Some connections will fail with "Thread limit exceeded"

### Test Speed Limit

Run high-throughput test:

```bash
# Monitor throughput in proxy logs
./target/release/test-client -p 127.0.0.1:8080 -u testuser -P testpass -c 10 -d 60
```

Check the reporter output for `current_throughput` - it should respect `max_throughput` setting.

### Test Bandwidth Limit

Run until quota exhausted:

```bash
# Keep running until bandwidth quota is reached
./target/release/test-client -p 127.0.0.1:8080 -u testuser -P testpass -c 5 -d 300
```

Connections will fail with "Bandwidth quota exceeded" once the limit is hit.

## 7. Monitor

Watch the logs for:

- 🔄 User sync events (every `update-interval` seconds)
- 📊 Statistics collection
- 🎯 Provider selection
- 📡 Connection handling
- ⚠️ Limit violations

```bash
# Run with JSON logs
RUST_LOG=debug ./target/release/riptide --config config.json | jq
```

## Troubleshooting

### "No upstream providers configured"

Check your `config.json` has at least one entry in the `upstream` array.

### "Invalid credentials"

Make sure:
1. Control plane API is running and returning users
2. The `api-key` in config.json matches your API
3. User credentials are synced (wait for first sync cycle)

### "Connection rate limit exceeded"

Too many connections too fast. The proxy limits 100 connections/second per user by default.

### io_uring errors (Linux only)

If you see io_uring errors, the proxy will automatically fall back to buffered copy. This is normal on:
- Older kernels (<5.10)
- Systems with io_uring disabled
- Non-Linux systems

## Performance Tuning

### For High Throughput

1. Increase system file descriptor limits:
```bash
ulimit -n 1000000
```

2. Enable io_uring (Linux only):
```bash
# Check kernel support
uname -r  # Should be 5.10+
```

3. Use multiple ports:
```json
{
  "ports": {
    "userpass": "8080-8089"
  }
}
```

### For Low Memory

1. Reduce update interval:
```json
{
  "server": {
    "update-interval": 30
  }
}
```

2. Use fewer concurrent connections per test

## Next Steps

1. ✅ Test with real upstream providers
2. ✅ Configure proper limits in control plane
3. ✅ Run load tests to verify performance
4. ✅ Monitor CPU and RAM usage
5. ✅ Verify parameter mapping works correctly

## Support

For issues or questions, refer to:
- `README.md` - Full documentation
- `config.example.json` - Configuration examples
- PRD in `../e:\JOB\prd.md` - Requirements specification

