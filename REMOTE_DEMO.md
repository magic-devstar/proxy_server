# Remote Demonstration Guide - Riptide Rust Proxy MVP

This guide explains how to demonstrate the Riptide proxy to a remote client.

## 🚀 Quick Demo Options

### Option 1: ngrok Tunnel (Fastest - 5 minutes)

**Best for:** Quick demos, testing, no server setup needed

#### Steps:

1. **Start the proxy locally:**
```bash
cd rust
./target/release/riptide --config config.json
```

2. **Install ngrok:**
```bash
# macOS
brew install ngrok

# Linux/Windows
# Download from https://ngrok.com/download
```

3. **Create tunnel:**
```bash
ngrok tcp 8080
```

4. **Share with client:**
```
Forwarding: tcp://0.tcp.ngrok.io:12345 -> localhost:8080
```

#### Client Demo Commands:

```bash
# Basic test
curl -x http://testuser:testpass@0.tcp.ngrok.io:12345 https://httpbin.org/ip

# With parameter mapping
curl -x http://testuser-country-us-session-demo:testpass@0.tcp.ngrok.io:12345 https://httpbin.org/ip

# Load test
./test-client --proxy 0.tcp.ngrok.io:12345 -u testuser -P testpass -c 50 -d 30
```

**Pros:**
- ✅ No server setup required
- ✅ Instant deployment
- ✅ Works from any location

**Cons:**
- ❌ Free tier has connection limits
- ❌ URLs change each restart
- ❌ Not suitable for production

---

### Option 2: Cloud Server Deployment (Recommended)

**Best for:** Professional demos, client testing, production-like environment

#### A. AWS EC2

```bash
# 1. Launch EC2 instance
# - AMI: Ubuntu 22.04 LTS
# - Instance type: t3.medium (2 vCPU, 4GB RAM)
# - Security Group: Allow TCP 8080 from client IP

# 2. Connect via SSH
ssh -i your-key.pem ubuntu@ec2-xx-xx-xx-xx.compute.amazonaws.com

# 3. Install dependencies
sudo apt update
sudo apt install -y build-essential curl

# 4. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 5. Upload and build
scp -i your-key.pem -r rust/ ubuntu@ec2-xx-xx-xx-xx.compute.amazonaws.com:~/
ssh -i your-key.pem ubuntu@ec2-xx-xx-xx-xx.compute.amazonaws.com
cd rust
cargo build --release

# 6. Configure
cp config.example.json config.json
nano config.json

# 7. Run
./target/release/riptide --config config.json
```

#### B. DigitalOcean Droplet

```bash
# 1. Create Droplet
# - Image: Ubuntu 22.04
# - Plan: Basic - $12/mo (2GB RAM)
# - Firewall: Allow port 8080

# 2. SSH and setup (same as AWS above)
ssh root@your-droplet-ip
```

#### C. Google Cloud Platform

```bash
# 1. Create Compute Engine VM
# - OS: Ubuntu 22.04 LTS
# - Machine type: e2-medium
# - Firewall: Allow tcp:8080

# 2. SSH and setup
gcloud compute ssh your-vm-name
# Follow same setup as AWS
```

#### Client Demo:

```bash
# Replace YOUR_SERVER_IP with actual IP
export PROXY_IP=xx.xx.xx.xx

# Basic test
curl -x http://testuser:testpass@$PROXY_IP:8080 https://httpbin.org/ip

# Parameter mapping test
curl -x http://testuser-country-us-city-newyork:testpass@$PROXY_IP:8080 https://httpbin.org/ip

# Load test
./test-client --proxy $PROXY_IP:8080 -u testuser -P testpass -c 100 -d 60
```

**Pros:**
- ✅ Professional setup
- ✅ Production-like environment
- ✅ Stable public IP
- ✅ Can handle high load

**Cons:**
- ❌ Costs money ($5-50/month)
- ❌ Takes 10-15 minutes to setup
- ❌ Requires cloud account

---

### Option 3: Docker Deployment

**Best for:** Containerized environments, easy deployment, consistent setup

#### Setup:

```bash
# 1. Build Docker image
docker build -t riptide-proxy .

# 2. Run container
docker run -d \
  --name riptide \
  -p 8080:8080 \
  -v $(pwd)/config.json:/etc/riptide/config.json:ro \
  -v $(pwd)/logs:/var/log/riptide \
  riptide-proxy

# Or use docker-compose
docker-compose up -d
```

#### Deploy to Cloud:

```bash
# AWS ECS / Google Cloud Run / Azure Container Instances
docker tag riptide-proxy your-registry/riptide-proxy:latest
docker push your-registry/riptide-proxy:latest
```

**Pros:**
- ✅ Portable and consistent
- ✅ Easy to deploy anywhere
- ✅ Scalable

**Cons:**
- ❌ Requires Docker knowledge
- ❌ Additional layer of complexity

---

### Option 4: Fly.io / Railway (Easiest Cloud)

**Best for:** Minimal setup, automatic HTTPS, global deployment

#### Fly.io:

```bash
# 1. Install flyctl
curl -L https://fly.io/install.sh | sh

# 2. Login
fly auth login

# 3. Create fly.toml
fly launch

# 4. Deploy
fly deploy
```

#### Railway:

```bash
# 1. Install Railway CLI
npm install -g @railway/cli

# 2. Login
railway login

# 3. Deploy
railway up
```

**Pros:**
- ✅ Extremely easy
- ✅ Automatic SSL/TLS
- ✅ Global CDN
- ✅ Free tier available

**Cons:**
- ❌ Platform-specific
- ❌ May have usage limits

---

## 📋 Demo Preparation Checklist

Before demonstrating to client:

### Technical Setup
- [ ] Proxy built and tested locally
- [ ] Config file with real upstream providers
- [ ] Mock API server running (or real control plane)
- [ ] Test users configured with appropriate limits
- [ ] Firewall rules configured (if using cloud)
- [ ] SSL certificates (if using HTTPS upstream)

### Demo Materials
- [ ] Test credentials prepared
- [ ] Demo script with curl commands
- [ ] Test client binary available
- [ ] Performance benchmarks ready
- [ ] Documentation links shared

### Test Scenarios
- [ ] Basic connectivity test
- [ ] Parameter mapping demonstration
- [ ] Thread limit enforcement
- [ ] Speed limit demonstration
- [ ] Bandwidth quota test
- [ ] Load test (multiple connections)
- [ ] SOCKS5 protocol test

---

## 🎬 Demo Script for Client

### 1. Basic Connectivity (1 minute)

```bash
# Show the proxy is running and accessible
curl -x http://testuser:testpass@PROXY_IP:8080 https://httpbin.org/ip
```

**Expected:** Returns IP address from upstream provider

### 2. Parameter Mapping (2 minutes)

```bash
# Demonstrate country routing
curl -x http://testuser-country-us:testpass@PROXY_IP:8080 https://httpbin.org/ip

# Demonstrate session sticky routing
curl -x http://testuser-session-demo123:testpass@PROXY_IP:8080 https://httpbin.org/ip

# Multiple parameters
curl -x http://testuser-country-uk-city-london:testpass@PROXY_IP:8080 https://httpbin.org/ip
```

**Expected:** Each request shows different IP based on parameters

### 3. Thread Limit Enforcement (3 minutes)

```bash
# User has max_threads = 10
# Try to open 20 concurrent connections

./test-client --proxy PROXY_IP:8080 -u limited_user -P testpass -c 20 -d 30
```

**Expected:** Only 10 connections succeed, rest fail with "Thread limit exceeded"

### 4. Speed Limit (2 minutes)

```bash
# User has max_throughput = 50 Mbps
# Run throughput test

./test-client --proxy PROXY_IP:8080 -u limited_user -P testpass -c 10 -d 60
```

**Expected:** Throughput stays at ~50 Mbps regardless of client capacity

### 5. Load Test (5 minutes)

```bash
# Demonstrate handling 100 concurrent connections
./test-client --proxy PROXY_IP:8080 -u testuser -P testpass -c 100 -d 60
```

**Expected:** 
- All 100 connections succeed
- Consistent throughput
- Low latency

### 6. SOCKS5 Protocol (1 minute)

```bash
# Test SOCKS5 support
./test-client --proxy PROXY_IP:8080 -u testuser -P testpass --socks5 -c 10 -d 10
```

**Expected:** SOCKS5 connections work correctly

### 7. Statistics & Monitoring (2 minutes)

```bash
# Show proxy logs (run on server)
tail -f /opt/riptide/logs/riptide.log | jq

# Show real-time statistics
# Check control plane API for reported stats
```

**Expected:**
- Real-time connection stats
- Bandwidth usage
- Throughput metrics
- User activity

---

## 📊 Performance Metrics to Highlight

### Throughput
```
Single connection: 100-500 Mbps
100 concurrent: 1+ Gbps aggregate
```

### Latency
```
Connection setup: <50ms
Per-request overhead: <5ms
```

### Resource Usage
```
Memory: ~4-32KB per connection (io_uring vs buffered)
CPU: <1% per 100 idle connections
CPU: 5-20% per 100 active connections
```

### Scalability
```
Max connections per GB RAM: ~1,000
Recommended: 1K req/sec per GB RAM
```

---

## 🔧 Troubleshooting During Demo

### Connection Refused
```bash
# Check proxy is running
ps aux | grep riptide

# Check port is listening
netstat -tlnp | grep 8080

# Check firewall
sudo ufw status
```

### Authentication Failed
```bash
# Verify user sync completed
grep "User sync completed" /opt/riptide/logs/riptide.log

# Check user credentials in config
```

### Slow Performance
```bash
# Check upstream provider latency
curl -x http://upstream.com:10000 https://httpbin.org/ip

# Check system resources
htop
iotop
```

---

## 📱 Client Requirements

Share these requirements with your client before the demo:

### For curl testing:
- curl installed
- Basic terminal access

### For test-client:
- Download test-client binary: [provide link]
- Or build from source: `cargo build --release --bin test-client`

### For browser testing:
- Configure proxy in browser settings:
  - HTTP Proxy: PROXY_IP
  - Port: 8080
  - Username: testuser
  - Password: testpass

---

## 🎁 Post-Demo Materials

After successful demo, provide:

1. **Access credentials** (if leaving demo server running)
2. **Documentation links:**
   - README.md
   - QUICKSTART.md
   - DEPLOYMENT.md
   - MVP_DELIVERY.md

3. **Test results:**
   - Load test reports
   - Performance benchmarks
   - Resource usage graphs

4. **Next steps:**
   - Production deployment timeline
   - Configuration requirements
   - Training schedule

---

## 💡 Pro Tips

### 1. Pre-test Everything
Run the full demo script yourself before showing the client.

### 2. Have Backup Plans
- If ngrok fails, have cloud instance ready
- If one upstream fails, have backup configured
- Keep logs tailing in separate terminal

### 3. Show Real Metrics
- Use `htop` to show resource usage
- Use `iftop` to show bandwidth
- Show control plane API statistics

### 4. Handle Questions
Common questions and answers:
- **"Can it handle more load?"** - Show scaling options
- **"What about security?"** - Show authentication, encryption
- **"How do we monitor it?"** - Show logging, statistics API
- **"What if upstream fails?"** - Show multiple upstream config

### 5. Record the Demo
```bash
# Screen recording for reference
asciinema rec demo-session.cast
```

---

## 📞 Support During Demo

If issues arise during the demo:

1. **Check logs immediately:**
   ```bash
   tail -n 50 /opt/riptide/logs/riptide.log
   ```

2. **Verify configuration:**
   ```bash
   cat /etc/riptide/config.json | jq
   ```

3. **Test upstream directly:**
   ```bash
   curl -x http://upstream.com:10000 https://httpbin.org/ip
   ```

4. **Have fallback demo ready:**
   - Pre-recorded video
   - Screenshots of successful tests
   - Performance graphs

---

## ✅ Success Criteria

Demo is successful when client sees:

- [x] Proxy accepting connections
- [x] Parameter mapping working correctly
- [x] Thread limits enforced
- [x] Speed limits enforced
- [x] Load handling (100+ concurrent connections)
- [x] Statistics reporting working
- [x] Low latency and high throughput
- [x] Stable operation under load

---

**Good luck with your demo! 🚀**

