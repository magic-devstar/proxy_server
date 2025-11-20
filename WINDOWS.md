# Windows Setup Guide

## Note on io_uring

io_uring is a Linux-specific feature and is **not available on Windows**. The proxy will automatically use the high-performance buffered copy path on Windows, which is still very efficient.

## Prerequisites

1. **Install Rust:** https://rustup.rs/
   - Download and run `rustup-init.exe`
   - Follow the installer prompts
   - Restart your terminal after installation

2. **Verify Installation:**
   ```cmd
   rustc --version
   cargo --version
   ```

## Building

### Option 1: Using build.bat

```cmd
cd rust
build.bat
```

### Option 2: Manual build

```cmd
cd rust
cargo build --release --no-default-features
```

Binaries will be in:
- `target\release\riptide.exe`
- `target\release\test-client.exe`

## Configuration

1. Copy example config:
   ```cmd
   copy config.example.json config.json
   ```

2. Edit `config.json` with your settings:
   - Update `api` section with your control plane URL and API key
   - Update `upstream` section with your upstream provider details

## Running

### Option 1: Using run.bat

```cmd
run.bat config.json
```

### Option 2: Manual run

```cmd
target\release\riptide.exe --config config.json
```

## Testing

### Using Test Client

```cmd
REM Single connection test
target\release\test-client.exe ^
  --proxy 127.0.0.1:8080 ^
  --username testuser ^
  --password testpass ^
  --target httpbin.org:80 ^
  --connections 1 ^
  --duration 5

REM Load test with 100 connections
target\release\test-client.exe ^
  --proxy 127.0.0.1:8080 ^
  --username testuser ^
  --password testpass ^
  --connections 100 ^
  --duration 30
```

### Mock API Server

1. **Install Python 3:** https://www.python.org/downloads/

2. **Install Flask:**
   ```cmd
   pip install flask
   ```

3. **Run Mock Server:**
   ```cmd
   python mock_api_server.py
   ```

## Using with curl (Windows)

Download curl for Windows or use Git Bash:

```bash
# HTTP CONNECT
curl -x http://testuser:testpass@127.0.0.1:8080 https://httpbin.org/ip

# With parameter mapping
curl -x http://testuser-country-us-session-abc:testpass@127.0.0.1:8080 https://httpbin.org/ip
```

## Monitoring

### View Logs

Logs will be printed to the console. To save to a file:

```cmd
target\release\riptide.exe --config config.json > riptide.log 2>&1
```

### Check Process

```cmd
REM CPU and memory usage
tasklist /FI "IMAGENAME eq riptide.exe" /V

REM Network connections
netstat -an | findstr :8080
```

## Common Issues

### Issue: "cargo: command not found"

**Solution:** Restart your terminal after installing Rust, or manually add Rust to PATH:
```cmd
set PATH=%PATH%;%USERPROFILE%\.cargo\bin
```

### Issue: Build errors about missing dependencies

**Solution:** Update Rust toolchain:
```cmd
rustup update
```

### Issue: Proxy won't start - "Address already in use"

**Solution:** Check if port is in use:
```cmd
netstat -an | findstr :8080
```

Kill the process using the port or change the port in config.json.

### Issue: "Failed to fetch users"

**Causes:**
1. Mock API server not running
2. Wrong API URL in config.json
3. Firewall blocking localhost connections

**Solution:**
1. Start mock API server: `python mock_api_server.py`
2. Check URL in config: `http://127.0.0.1:8000/api`
3. Allow through Windows Firewall if needed

## Windows Service (Optional)

To run as a Windows service, use NSSM (Non-Sucking Service Manager):

1. **Download NSSM:** https://nssm.cc/download

2. **Install Service:**
   ```cmd
   nssm install Riptide "C:\path\to\rust\target\release\riptide.exe"
   nssm set Riptide AppDirectory "C:\path\to\rust"
   nssm set Riptide AppParameters "--config config.json"
   nssm set Riptide DisplayName "Riptide Rust Proxy"
   nssm set Riptide Description "High-performance proxy server"
   nssm set Riptide Start SERVICE_AUTO_START
   ```

3. **Start Service:**
   ```cmd
   nssm start Riptide
   ```

4. **Check Status:**
   ```cmd
   nssm status Riptide
   ```

## Performance Notes

- Windows does **not** support io_uring (Linux-only)
- The buffered copy path is used instead
- Performance is still excellent (slightly lower than Linux io_uring)
- Expect 80-90% of Linux io_uring performance

## Limitations on Windows

- No io_uring zero-copy (uses buffered copy)
- Service management different from Linux (use NSSM)
- Some shell scripts won't work (use .bat versions)

## Recommended for Production

For best performance in production, deploy on Linux with kernel ≥5.10 to take advantage of io_uring zero-copy.

Windows is suitable for:
- Development and testing
- Non-critical deployments
- Environments where Linux is not available

---

For more information, see:
- README.md - Main documentation
- QUICKSTART.md - Quick start guide  
- TESTING.md - Testing guide
- DEPLOYMENT.md - Production deployment (Linux)

