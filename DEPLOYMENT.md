# Deployment Guide - Riptide Rust MVP

## System Requirements

### Minimum
- **OS:** Linux (Ubuntu 20.04+, Debian 11+, RHEL 8+)
- **CPU:** 2 cores
- **RAM:** 2 GB
- **Disk:** 500 MB
- **Network:** 100 Mbps

### Recommended for Production
- **OS:** Linux with kernel ≥5.10 (for io_uring)
- **CPU:** 8+ cores
- **RAM:** 16 GB
- **Disk:** 10 GB SSD
- **Network:** 1+ Gbps

### Software Dependencies
- Rust toolchain 1.70+ (for building)
- glibc 2.31+ or musl
- Linux kernel 5.10+ (for io_uring support)

## Building for Production

### Option 1: Native Build

```bash
cd rust

# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Build release binary
cargo build --release

# Binary is at: target/release/riptide
```

### Option 2: Static Binary (portable)

```bash
# Install musl target
rustup target add x86_64-unknown-linux-musl

# Build static binary
cargo build --release --target x86_64-unknown-linux-musl

# Binary is at: target/x86_64-unknown-linux-musl/release/riptide
```

### Option 3: Cross-compilation

```bash
# From development machine
cargo install cross

# Build for target platform
cross build --release --target x86_64-unknown-linux-gnu
```

## Installation

### 1. Create Deployment Directory

```bash
sudo mkdir -p /opt/riptide
sudo mkdir -p /opt/riptide/logs
sudo mkdir -p /etc/riptide
```

### 2. Copy Binary

```bash
# Native build
sudo cp target/release/riptide /opt/riptide/

# Static build
sudo cp target/x86_64-unknown-linux-musl/release/riptide /opt/riptide/

# Set permissions
sudo chmod +x /opt/riptide/riptide
```

### 3. Create Configuration

```bash
sudo nano /etc/riptide/config.json
```

```json
{
  "server": {
    "logging": "info",
    "inflation": 0,
    "sni-check": false,
    "node-name": "prod-node-01",
    "update-interval": 10,
    "source-ips": [],
    "retries": {
      "max-retries": 5,
      "timeout": 5
    }
  },
  "api": [{
    "name": "production-api",
    "base-url": "https://api.yourcompany.com/api",
    "api-key": "your-production-api-key",
    "default-package": "residential",
    "legacy": false,
    "ports": {
      "userpass": "8080-8089"
    }
  }],
  "upstream": [
    {
      "name": "provider-1",
      "ips": [
        "http://upstream1.provider.com:10000",
        "http://upstream2.provider.com:10000"
      ],
      "user": "your-upstream-user",
      "password": "your-upstream-password",
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
      "package": "residential",
      "allowed-countries": []
    }
  ],
  "clickhouse": {
    "host": "",
    "username": "",
    "password": ""
  }
}
```

### 4. Create Systemd Service

```bash
sudo nano /etc/systemd/system/riptide.service
```

```ini
[Unit]
Description=Riptide Rust Proxy
After=network.target

[Service]
Type=simple
User=riptide
Group=riptide
WorkingDirectory=/opt/riptide
ExecStart=/opt/riptide/riptide --config /etc/riptide/config.json
Restart=always
RestartSec=5
StandardOutput=append:/opt/riptide/logs/riptide.log
StandardError=append:/opt/riptide/logs/riptide.error.log

# Resource limits
LimitNOFILE=1048576
LimitNPROC=512

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/riptide/logs

[Install]
WantedBy=multi-user.target
```

### 5. Create User

```bash
sudo useradd -r -s /bin/false riptide
sudo chown -R riptide:riptide /opt/riptide
sudo chown -R riptide:riptide /etc/riptide
```

### 6. Configure System Limits

```bash
sudo nano /etc/security/limits.conf
```

Add:
```
riptide soft nofile 1048576
riptide hard nofile 1048576
riptide soft nproc 512
riptide hard nproc 512
```

```bash
sudo nano /etc/sysctl.conf
```

Add:
```
# Network tuning for high-performance proxy
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 8192
net.ipv4.ip_local_port_range = 1024 65535
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 30

# Increase buffer sizes
net.core.rmem_max = 134217728
net.core.wmem_max = 134217728
net.ipv4.tcp_rmem = 4096 87380 67108864
net.ipv4.tcp_wmem = 4096 65536 67108864

# Enable BBR congestion control (kernel 4.9+)
net.core.default_qdisc = fq
net.ipv4.tcp_congestion_control = bbr
```

Apply:
```bash
sudo sysctl -p
```

### 7. Start Service

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable on boot
sudo systemctl enable riptide

# Start service
sudo systemctl start riptide

# Check status
sudo systemctl status riptide
```

## Monitoring

### Service Status

```bash
# Check if running
sudo systemctl status riptide

# View logs
sudo journalctl -u riptide -f

# View structured logs
sudo tail -f /opt/riptide/logs/riptide.log | jq
```

### System Metrics

```bash
# CPU and RAM usage
ps aux | grep riptide

# Network connections
ss -tn | grep :8080 | wc -l

# File descriptors
lsof -u riptide | wc -l

# Bandwidth usage
iftop -i eth0
```

### Health Check

```bash
# Test connection
curl -x http://testuser:testpass@localhost:8080 https://httpbin.org/ip

# Check with test client
/opt/riptide/test-client --proxy localhost:8080 -u testuser -P testpass -c 1 -d 5
```

## Scaling

### Vertical Scaling

Adjust limits based on available RAM:

| RAM | Max Connections | Recommended update-interval |
|-----|-----------------|----------------------------|
| 2 GB | 5K | 30s |
| 4 GB | 10K | 20s |
| 8 GB | 20K | 15s |
| 16 GB | 50K | 10s |
| 32 GB | 100K | 10s |

### Horizontal Scaling

Run multiple instances:

```bash
# Node 1
sudo nano /etc/riptide/config.json
# Set node-name: "node-01"
# Set ports: "8080-8089"

# Node 2
# Set node-name: "node-02"
# Set ports: "8090-8099"

# Load balancer (HAProxy, nginx, etc.)
```

### Port Range

For high connection counts, use port ranges:

```json
{
  "ports": {
    "userpass": "8080-8089"
  }
}
```

This starts 10 listeners (one per port).

## Backup & Recovery

### Configuration Backup

```bash
# Backup config
sudo cp /etc/riptide/config.json /etc/riptide/config.json.backup

# Automated backup
sudo crontab -e
# Add: 0 0 * * * cp /etc/riptide/config.json /backup/riptide-config-$(date +\%Y\%m\%d).json
```

### Log Rotation

```bash
sudo nano /etc/logrotate.d/riptide
```

```
/opt/riptide/logs/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0640 riptide riptide
    sharedscripts
    postrotate
        systemctl reload riptide > /dev/null 2>&1 || true
    endscript
}
```

## Security

### Firewall Rules

```bash
# Allow proxy ports (adjust as needed)
sudo ufw allow 8080:8089/tcp

# Allow only from specific IPs (recommended)
sudo ufw allow from 203.0.113.0/24 to any port 8080:8089 proto tcp

# Enable firewall
sudo ufw enable
```

### TLS/SSL (if needed)

For HTTPS API endpoints, ensure certificates are valid:

```bash
# Update CA certificates
sudo apt-get install ca-certificates
sudo update-ca-certificates
```

## Troubleshooting

### Service Won't Start

```bash
# Check logs
sudo journalctl -u riptide -n 100 --no-pager

# Check config syntax
/opt/riptide/riptide --config /etc/riptide/config.json
```

### High CPU Usage

- Check number of connections: `ss -tn | grep :8080 | wc -l`
- Reduce concurrent connections
- Check upstream provider latency

### High Memory Usage

- Check for memory leaks: `ps aux | grep riptide`
- Reduce update-interval
- Reduce number of concurrent connections

### Connection Failures

```bash
# Test upstream connectivity
curl -x http://your-upstream.com:10000 https://httpbin.org/ip

# Check DNS resolution
dig api.yourcompany.com

# Check firewall rules
sudo iptables -L -n
```

### io_uring Not Working

```bash
# Check kernel version
uname -r  # Should be ≥5.10

# Check io_uring support
zgrep CONFIG_IO_URING /proc/config.gz

# If not available, rebuild without io_uring
cargo build --release --no-default-features
```

## Upgrading

### Zero-Downtime Upgrade

```bash
# Build new version
cd /path/to/source/rust
cargo build --release

# Copy new binary with different name
sudo cp target/release/riptide /opt/riptide/riptide-new

# Test new binary
sudo -u riptide /opt/riptide/riptide-new --config /etc/riptide/config.json &
# Ctrl+C after verifying it starts

# Swap binaries
sudo systemctl stop riptide
sudo mv /opt/riptide/riptide /opt/riptide/riptide-old
sudo mv /opt/riptide/riptide-new /opt/riptide/riptide
sudo systemctl start riptide

# Check status
sudo systemctl status riptide

# If issues, rollback
# sudo systemctl stop riptide
# sudo mv /opt/riptide/riptide-old /opt/riptide/riptide
# sudo systemctl start riptide
```

## Maintenance

### Weekly Tasks

- Check logs for errors: `sudo journalctl -u riptide -p err -n 100`
- Verify statistics reporting: check control plane API
- Review resource usage: `htop`, `iotop`

### Monthly Tasks

- Review configuration for optimization
- Check for Rust/dependency updates
- Analyze performance metrics
- Plan capacity upgrades if needed

## Support Checklist

When reporting issues, provide:

- [ ] Rust version: `rustc --version`
- [ ] Binary build type: native/static/cross-compiled
- [ ] Kernel version: `uname -r`
- [ ] Config file (sanitized)
- [ ] Recent logs: `sudo journalctl -u riptide -n 500`
- [ ] Resource usage: `top`, `free -h`
- [ ] Network stats: `ss -s`
- [ ] Load test results

## Production Checklist

Before going live:

- [ ] Binary built with `--release` flag
- [ ] Config file validated and tested
- [ ] Systemd service installed and enabled
- [ ] System limits configured (ulimit, sysctl)
- [ ] Firewall rules applied
- [ ] Monitoring set up (logs, metrics)
- [ ] Backup strategy implemented
- [ ] Load testing completed successfully
- [ ] Documentation updated with node-specific info
- [ ] Team trained on operations

## Performance Tuning

### For Low Latency

```json
{
  "server": {
    "update-interval": 5,  // Faster sync
    "retries": {
      "timeout": 2  // Faster timeout
    }
  }
}
```

### For High Throughput

- Use multiple ports: `"8080-8099"`
- Increase system limits: `LimitNOFILE=2097152`
- Enable io_uring (Linux)
- Use BBR congestion control

### For Low Memory

```json
{
  "server": {
    "update-interval": 60  // Slower sync = less memory
  }
}
```

- Reduce concurrent connections
- Use port range of 1-2 ports only

---

**Questions?** Refer to README.md and TESTING.md for more details.

