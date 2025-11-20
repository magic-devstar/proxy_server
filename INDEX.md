# 📖 Riptide Rust MVP - Documentation Index

Welcome to the Riptide Rust Proxy MVP! This index helps you navigate the documentation and get started quickly.

## 🚀 Getting Started

**New to this project?** Start here:

1. **[MVP_DELIVERY.md](MVP_DELIVERY.md)** ⭐ START HERE
   - What was delivered
   - MVP acceptance criteria
   - Quick overview of all features

2. **[QUICKSTART.md](QUICKSTART.md)** ⚡
   - Get running in 5 minutes
   - Minimal configuration example
   - First test commands

3. **[README.md](README.md)** 📚
   - Complete documentation
   - Architecture overview
   - Feature details

## 🔧 Setup & Building

### Linux / macOS

- **Build:** Run `./build.sh`
- **Quick start:** Run `./run.sh config.json`
- See: [QUICKSTART.md](QUICKSTART.md)

### Windows

- **Build:** Run `build.bat`
- **Quick start:** Run `run.bat config.json`
- See: [WINDOWS.md](WINDOWS.md) for Windows-specific instructions

## 🧪 Testing

**Want to test the proxy?**

- **[TESTING.md](TESTING.md)** - Comprehensive testing guide
  - Mock API server setup
  - Automated test suite
  - Manual testing procedures
  - Load testing instructions
  - Performance verification

**Test Tools:**
- `test-client` binary - Load testing and verification
- `test.sh` - Automated test suite (Linux/macOS)
- `mock_api_server.py` - Mock control plane API

## 🌐 Production Deployment

**Ready to deploy?**

- **[DEPLOYMENT.md](DEPLOYMENT.md)** - Production deployment guide
  - System requirements
  - Installation steps
  - Systemd service setup
  - Security hardening
  - Monitoring and maintenance
  - Performance tuning

## 📋 Configuration

**Configuration examples:**
- `config.example.json` - Full example with comments
- `config.json` - Your active config (create from example)

**Configuration documentation:**
- See [README.md](README.md) - Configuration section
- See `../cmd/riptide/config.md` - Original config spec (Go version)

## 📦 Project Structure

```
rust/
├── src/                      # Source code
│   ├── main.rs              # Entry point
│   ├── config.rs            # Configuration
│   ├── limits.rs            # Limit enforcement
│   ├── upstream.rs          # Upstream selection
│   ├── proxy.rs             # Protocol handlers
│   ├── tunnel.rs            # Bidirectional copy
│   ├── stats.rs             # Statistics
│   └── bin/
│       └── test_client.rs   # Test client
│
├── target/                   # Build output
│   └── release/
│       ├── riptide          # Main binary
│       └── test-client      # Test binary
│
├── Cargo.toml               # Rust dependencies
├── Cargo.lock               # Dependency lock file
├── .gitignore               # Git ignore rules
│
├── *.md                     # Documentation
├── *.sh                     # Linux/macOS scripts
├── *.bat                    # Windows scripts
├── *.py                     # Python utilities
│
└── config.example.json      # Configuration example
```

## 📚 Documentation Files

### Essential Docs (Read These)

| File | Purpose | When to Read |
|------|---------|--------------|
| **MVP_DELIVERY.md** | What was delivered, acceptance criteria | START HERE |
| **QUICKSTART.md** | Get running in 5 minutes | First time setup |
| **README.md** | Complete documentation | Understanding the system |

### Specialized Docs

| File | Purpose | When to Read |
|------|---------|--------------|
| **TESTING.md** | Testing guide | Before testing |
| **DEPLOYMENT.md** | Production deployment | Before deploying |
| **WINDOWS.md** | Windows-specific setup | If using Windows |
| **INDEX.md** | This file | Navigation |

## 🛠️ Build Scripts

### Linux / macOS

- **build.sh** - Build the project
- **run.sh** - Quick start the proxy
- **test.sh** - Run automated tests

### Windows

- **build.bat** - Build the project
- **run.bat** - Quick start the proxy

### Cross-Platform

- **mock_api_server.py** - Mock control plane (requires Python + Flask)

## 🎯 Common Tasks

### "I want to build and run the proxy"

**Linux/macOS:**
```bash
./build.sh
cp config.example.json config.json
# Edit config.json
./run.sh config.json
```

**Windows:**
```cmd
build.bat
copy config.example.json config.json
REM Edit config.json
run.bat config.json
```

### "I want to test if it's working"

**Quick test with curl:**
```bash
curl -x http://testuser:testpass@127.0.0.1:8080 https://httpbin.org/ip
```

**Full test suite:**
```bash
# Terminal 1: Start mock API
python3 mock_api_server.py

# Terminal 2: Start proxy
./run.sh config.json

# Terminal 3: Run tests
./test.sh  # Linux/macOS
```

### "I want to test parameter mapping"

```bash
# Country parameter
curl -x http://testuser-country-us:testpass@127.0.0.1:8080 https://httpbin.org/ip

# Session parameter
curl -x http://testuser-session-abc123:testpass@127.0.0.1:8080 https://httpbin.org/ip

# Multiple parameters
curl -x http://testuser-country-uk-city-london:testpass@127.0.0.1:8080 https://httpbin.org/ip
```

Check proxy logs for: `🎯 Selected provider: ... user: upstream-user-country-uk-city-london`

### "I want to load test"

```bash
# 100 concurrent connections, 60 seconds
./target/release/test-client \
  --proxy 127.0.0.1:8080 \
  --username testuser \
  --password testpass \
  --connections 100 \
  --duration 60
```

See [TESTING.md](TESTING.md) for more load testing scenarios.

### "I want to verify limits are working"

See [TESTING.md](TESTING.md) sections:
- Test 4: Thread Limit
- Test 5: Speed Limit
- Test 6: Bandwidth Quota

### "I want to deploy to production"

1. Read [DEPLOYMENT.md](DEPLOYMENT.md)
2. Build release binary
3. Create systemd service
4. Configure system limits
5. Start and monitor

## 🐛 Troubleshooting

### Build Issues

**"cargo: command not found"**
- Install Rust: https://rustup.rs/
- Restart terminal after installation

**"Failed to compile"**
- Update Rust: `rustup update`
- Check dependencies: `cargo update`

### Runtime Issues

**"No upstream providers configured"**
- Check `config.json` has valid upstream entries

**"Invalid credentials"**
- Ensure mock API server is running (for testing)
- Wait for first user sync (10 seconds)
- Check username/password

**"Connection refused"**
- Check proxy is running: `ps aux | grep riptide` (Linux) or `tasklist | findstr riptide` (Windows)
- Check port is correct: default is 8080
- Check firewall settings

### Performance Issues

**High CPU usage**
- Reduce concurrent connections
- Check upstream provider latency
- Verify io_uring is working (Linux: check logs)

**High memory usage**
- Normal: ~32KB per connection
- Check for leaks: monitor over time
- Reduce connections if needed

See full troubleshooting guides in:
- [TESTING.md](TESTING.md) - Testing issues
- [DEPLOYMENT.md](DEPLOYMENT.md) - Production issues
- [WINDOWS.md](WINDOWS.md) - Windows-specific issues

## 📊 Feature Checklist

### MVP Requirements ✅

- ✅ Upstream connections with parameter mapping
- ✅ Speed limit (throughput limiting)
- ✅ Thread limit (concurrent connections)
- ✅ Bandwidth limit (quota tracking)
- ✅ IOuring zero-copy (Linux)
- ✅ Test client for verification
- ✅ Load testing capability

### Protocols ✅

- ✅ HTTP CONNECT (HTTPS tunneling)
- ✅ SOCKS5 TCP CONNECT
- ✅ Dual authentication support

### Limits ✅

- ✅ Per-user thread cap
- ✅ Per-user throughput cap
- ✅ Per-user bandwidth quota
- ✅ Connection rate limiting

### Background Tasks ✅

- ✅ User sync from control plane
- ✅ Statistics collection
- ✅ Periodic reporting

### Additional Features ✅

- ✅ Weighted provider selection
- ✅ Multiple IPs per provider
- ✅ Both HTTP and SOCKS5 upstreams
- ✅ Automatic fallback (io_uring → buffered)
- ✅ Cross-platform support

## 🔗 Related Files

### Original Go Implementation

Reference files from the Go version (for comparison):
- `../cmd/riptide/config.json` - Example config
- `../cmd/riptide/config.md` - Config spec
- `../internal/proxytunnel/upstream/handle.go` - Original upstream handler

### PRD (Product Requirements)

- `../e:\JOB\prd.md` - Full product requirements document

## 📞 Support

### Documentation Priority

1. **Start here:** [MVP_DELIVERY.md](MVP_DELIVERY.md)
2. **Quick setup:** [QUICKSTART.md](QUICKSTART.md)
3. **Testing:** [TESTING.md](TESTING.md)
4. **Deep dive:** [README.md](README.md)
5. **Production:** [DEPLOYMENT.md](DEPLOYMENT.md)

### Getting Help

1. Check the relevant documentation file above
2. Look for similar issues in troubleshooting sections
3. Review logs for error messages
4. Check configuration is valid

### Reporting Issues

When reporting issues, include:
- [ ] Rust version: `rustc --version`
- [ ] Operating system and version
- [ ] Config file (sanitized)
- [ ] Complete error message
- [ ] Steps to reproduce
- [ ] Expected vs actual behavior

## ✅ Next Steps

1. ✅ Read [MVP_DELIVERY.md](MVP_DELIVERY.md) to understand what was delivered
2. ✅ Follow [QUICKSTART.md](QUICKSTART.md) to get running
3. ✅ Use [TESTING.md](TESTING.md) to verify everything works
4. ✅ Test with your actual upstream providers
5. ✅ Run load tests to verify performance
6. ✅ When satisfied, follow [DEPLOYMENT.md](DEPLOYMENT.md) for production

---

**Need more help?** Start with [MVP_DELIVERY.md](MVP_DELIVERY.md) which has a complete overview of the MVP and acceptance criteria.

**Ready to start?** Go to [QUICKSTART.md](QUICKSTART.md) to get running in 5 minutes!

