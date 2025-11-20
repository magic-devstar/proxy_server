# Testing Guide - Riptide Rust MVP

This guide shows you how to test all MVP features end-to-end.

## Setup

### 1. Build the Proxy

```bash
cd rust
./build.sh
```

### 2. Start Mock API Server (for testing)

```bash
# Install Flask if needed
pip3 install flask

# Start the mock API server
python3 mock_api_server.py
```

This starts a mock control plane API on `http://127.0.0.1:8000` with three test users:

| Username | Password | Max Threads | Max Speed | Max Bandwidth |
|----------|----------|-------------|-----------|---------------|
| testuser | testpass | 100 | 50 Mbps | 10 GB |
| premium | premium123 | 500 | 100 Mbps | 100 GB |
| limited | limited123 | 10 | 10 Mbps | 1 GB |

### 3. Configure Proxy

Create `config.json`:

```json
{
  "server": {
    "logging": "info",
    "inflation": 0,
    "sni-check": false,
    "node-name": "test-node",
    "update-interval": 10,
    "source-ips": [],
    "retries": { "max-retries": 3, "timeout": 5 }
  },
  "api": [{
    "name": "mock-api",
    "base-url": "http://127.0.0.1:8000/api",
    "api-key": "test-key-12345",
    "default-package": "residential",
    "legacy": false,
    "ports": { "userpass": "8080-8080" }
  }],
  "upstream": [{
    "name": "test-provider",
    "ips": ["http://your-upstream-proxy.com:10000"],
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
  }]
}
```

**Note:** Replace the upstream provider details with your actual upstream proxy.

### 4. Start the Proxy

```bash
./run.sh config.json
```

You should see:
```
🚀 Riptide Rust Proxy starting...
✅ Configuration loaded successfully
🌐 Started proxy server on port 8080
✅ All servers started successfully
🔄 Starting user sync...
✅ Fetched 3 users from mock-api
```

## Test Suite

### Automated Tests

Run the full test suite:

```bash
chmod +x test.sh
./test.sh
```

This runs 5 automated tests:
1. ✅ Single connection
2. ✅ Multiple connections (10)
3. ✅ SOCKS5 protocol
4. ✅ Parameter mapping
5. ✅ Load test (50 connections)

### Manual Tests

#### Test 1: Basic HTTP CONNECT

```bash
curl -v -x http://testuser:testpass@127.0.0.1:8080 https://httpbin.org/ip
```

**Expected:** Success, returns your proxy IP

#### Test 2: Parameter Mapping

```bash
# Test with country parameter
curl -x http://testuser-country-us:testpass@127.0.0.1:8080 https://httpbin.org/ip

# Test with session ID
curl -x http://testuser-session-abc123:testpass@127.0.0.1:8080 https://httpbin.org/ip

# Test with multiple parameters
curl -x http://testuser-country-uk-city-london-session-test:testpass@127.0.0.1:8080 https://httpbin.org/ip
```

**Expected:** 
- Proxy connects successfully
- Check proxy logs for: `🎯 Selected provider: test-provider, user: upstream-user-country-uk-city-london-session-test`

#### Test 3: Invalid Credentials

```bash
curl -x http://baduser:badpass@127.0.0.1:8080 https://httpbin.org/ip
```

**Expected:** `407 Proxy Authentication Required`

#### Test 4: Thread Limit

Test with the "limited" user (max 10 threads):

```bash
# Start 20 concurrent connections
./target/release/test-client \
  --proxy 127.0.0.1:8080 \
  --username limited \
  --password limited123 \
  --connections 20 \
  --duration 30
```

**Expected:**
- First 10 connections succeed
- Additional connections fail with "Thread limit exceeded"
- Check proxy logs for limit violations

#### Test 5: Speed Limit

Test throughput limiting with "limited" user (10 Mbps):

```bash
# Run high-volume test
./target/release/test-client \
  --proxy 127.0.0.1:8080 \
  --username limited \
  --password limited123 \
  --connections 5 \
  --duration 60
```

**Expected:**
- Throughput capped around 10 Mbps
- Check reporter output (in proxy logs after 10 seconds): `current_throughput` should be ≤ 10 MB/s

#### Test 6: Bandwidth Quota

Test quota exhaustion:

```bash
# Run until 1GB quota exhausted (limited user)
./target/release/test-client \
  --proxy 127.0.0.1:8080 \
  --username limited \
  --password limited123 \
  --connections 10 \
  --duration 600
```

**Expected:**
- Connections work initially
- After ~1GB transferred, connections fail with "Bandwidth quota exceeded"
- Mock API server shows updated bytes_used

#### Test 7: SOCKS5 Protocol

```bash
# HTTP CONNECT vs SOCKS5
./target/release/test-client \
  --proxy 127.0.0.1:8080 \
  --username testuser \
  --password testpass \
  --connections 10 \
  --duration 10

./target/release/test-client \
  --proxy 127.0.0.1:8080 \
  --username testuser \
  --password testpass \
  --socks5 \
  --connections 10 \
  --duration 10
```

**Expected:** Both work identically

#### Test 8: Concurrent Users

Run tests with different users simultaneously:

```bash
# Terminal 1: testuser
./target/release/test-client -p 127.0.0.1:8080 -u testuser -P testpass -c 20 -d 60 &

# Terminal 2: premium
./target/release/test-client -p 127.0.0.1:8080 -u premium -P premium123 -c 50 -d 60 &

# Terminal 3: limited
./target/release/test-client -p 127.0.0.1:8080 -u limited -P limited123 -c 5 -d 60 &
```

**Expected:**
- Each user gets independent limits
- Statistics reported separately for each key
- No interference between users

#### Test 9: Load Test

Stress test with premium user:

```bash
./target/release/test-client \
  --proxy 127.0.0.1:8080 \
  --username premium \
  --password premium123 \
  --connections 500 \
  --duration 60
```

**Expected:**
- All 500 connections succeed (user has max_threads = 500)
- Monitor CPU and RAM usage
- Check for memory leaks (RAM should stabilize)

#### Test 10: io_uring Verification (Linux only)

If on Linux with io_uring support, verify zero-copy is working:

```bash
# Run test and check logs
RUST_LOG=debug ./target/release/riptide --config config.json 2>&1 | grep -i uring
```

**Expected:**
- Logs should show io_uring being used
- If it falls back to buffered copy, check kernel version (`uname -r` should be ≥5.10)

## Monitoring

### Watch Live Statistics

```bash
# In another terminal, watch the mock API server output
# Every 10 seconds you should see:

POST /api/riptide/report
  Reports: 3 entries
    - residential:testuser:5001:1001: 52428800 bytes, 5 threads, 5.24 MB/s
    - residential:premium:5002:1002: 104857600 bytes, 50 threads, 10.49 MB/s
    - residential:limited:5003:1003: 10485760 bytes, 5 threads, 1.05 MB/s
```

### Check Proxy Logs

```bash
# Watch proxy logs
tail -f /path/to/proxy/output | jq
```

Look for:
- `🔄 Starting user sync...` - Background sync running
- `✅ Fetched N users` - Users loaded successfully
- `📊 Collecting statistics...` - Reporter active
- `🎯 Selected provider` - Upstream selection working
- `📡 CONNECT to X` - Connections being handled

## Troubleshooting

### Issue: "No upstream providers configured"

**Solution:** Check `config.json` has valid upstream entries.

### Issue: "Invalid credentials"

**Causes:**
1. Mock API server not running
2. User sync hasn't completed yet (wait 10 seconds)
3. Wrong username/password

**Solution:** 
```bash
# Check API server is running
curl http://127.0.0.1:8000/health

# Check users endpoint
curl -H "api-key: test-key-12345" http://127.0.0.1:8000/api/riptide
```

### Issue: "Connection rate limit exceeded"

**Cause:** Too many new connections per second (>100/sec per user)

**Solution:** Add delay between connections or use multiple users

### Issue: "Thread limit exceeded"

**Expected behavior** when exceeding user's max_threads. Verify limits in mock API server output.

### Issue: io_uring errors

**Cause:** Kernel doesn't support io_uring

**Solution:** Proxy automatically falls back to buffered copy. This is normal and still performant.

### Issue: High CPU usage

**Possible causes:**
1. Too many connections for hardware
2. Upstream provider slow (blocking connections)
3. Rate limiting causing busy waiting

**Solution:**
1. Reduce concurrent connections
2. Check upstream latency
3. Increase update-interval in config

### Issue: Memory leak

**Debug:**
```bash
# Monitor memory over time
watch -n 1 'ps aux | grep riptide'

# Run extended test
./target/release/test-client -c 100 -d 3600
```

**Expected:** Memory should stabilize after initial ramp-up (buffers allocated).

## Performance Benchmarks

Target metrics for different setups:

| RAM | Connections | Expected Throughput | CPU Usage |
|-----|-------------|---------------------|-----------|
| 1 GB | 1K | ~100 Mbps | <20% |
| 2 GB | 5K | ~500 Mbps | <30% |
| 4 GB | 10K | ~1 Gbps | <40% |
| 8 GB | 20K | ~2 Gbps | <50% |
| 16 GB | 50K | ~5 Gbps | <60% |

**Actual performance depends on:**
- Upstream provider latency
- Network bandwidth
- io_uring support
- Kernel version
- CPU speed

## Success Criteria

✅ **MVP is working if:**

1. ✅ Proxy accepts HTTP CONNECT and SOCKS5 connections
2. ✅ Authentication works (users from API)
3. ✅ Parameter mapping builds correct upstream credentials
4. ✅ Thread limits are enforced (connections rejected when exceeded)
5. ✅ Speed limits are enforced (throughput capped)
6. ✅ Bandwidth limits are enforced (quota tracking)
7. ✅ Statistics are collected and reported every interval
8. ✅ Load test with 100+ connections succeeds
9. ✅ CPU and RAM usage are reasonable
10. ✅ No crashes or memory leaks

## Next Steps

After successful testing:

1. ✅ Deploy with real upstream providers
2. ✅ Connect to production control plane API
3. ✅ Run extended load tests (24+ hours)
4. ✅ Monitor in production
5. ✅ Tune limits based on hardware
6. ✅ Add monitoring/alerting
7. ✅ Implement remaining PRD features (if needed)

