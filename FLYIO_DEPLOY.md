# 🚀 Fly.io Deployment Guide - Riptide Proxy Demo

Deploy your Riptide proxy to Fly.io in 10 minutes for remote client demonstration.

## Prerequisites

- Fly.io account (sign up at https://fly.io/app/sign-up - free tier available)
- Your upstream provider credentials
- Your control plane API URL (or use mock API)

## Step 1: Install Fly.io CLI

### Windows (PowerShell):
```powershell
iwr https://fly.io/install.ps1 -useb | iex
```

### macOS/Linux:
```bash
curl -L https://fly.io/install.sh | sh
```

## Step 2: Login to Fly.io

```bash
fly auth login
```

This will open a browser window for authentication.

## Step 3: Configure Your Application

### A. Edit Configuration

Update `config.demo.json` with your actual settings:

```powershell
# Copy demo config to config.json
cp config.demo.json config.json

# Edit with your settings
notepad config.json
```

**Important settings to update:**
- `api[0].base-url` - Your control plane API URL
- `api[0].api-key` - Your API key
- `upstream[0].ips` - Your upstream provider URLs
- `upstream[0].user` - Your upstream username
- `upstream[0].password` - Your upstream password

### B. Update Dockerfile to Include Config

The Dockerfile is already set up. Just make sure `config.json` exists before deploying.

## Step 4: Launch Application

```bash
# Initialize Fly.io app (first time only)
fly launch --no-deploy

# When prompted:
# - App name: riptide-demo (or your choice)
# - Region: Choose closest to your client
# - PostgreSQL: No
# - Redis: No
```

This creates a `fly.toml` file (already provided).

## Step 5: Set Secrets (Optional)

If you want to keep sensitive data out of config.json:

```bash
fly secrets set API_KEY=your-api-key
fly secrets set UPSTREAM_USER=your-upstream-user
fly secrets set UPSTREAM_PASS=your-upstream-password
```

## Step 6: Deploy!

```bash
fly deploy
```

This will:
1. Build the Docker image
2. Upload to Fly.io
3. Start your application
4. Provide you with a public URL

## Step 7: Get Your Public URL

```bash
fly status
```

Look for the hostname, e.g., `riptide-demo.fly.dev`

## Step 8: Test Your Deployment

```bash
# Get your app URL
fly status

# Test from your local machine
curl -x http://testuser:testpass@riptide-demo.fly.dev:8080 https://httpbin.org/ip
```

## Step 9: Share with Client

Your proxy is now accessible at:
```
Host: riptide-demo.fly.dev (or your custom app name)
Port: 8080
Protocol: HTTP CONNECT or SOCKS5
```

### Client Test Commands

```bash
# Basic test
curl -x http://testuser:testpass@riptide-demo.fly.dev:8080 https://httpbin.org/ip

# With parameter mapping
curl -x http://testuser-country-us-session-abc:testpass@riptide-demo.fly.dev:8080 https://httpbin.org/ip

# Load test (if you have test-client)
./test-client --proxy riptide-demo.fly.dev:8080 -u testuser -P testpass -c 50 -d 30
```

## Monitoring & Logs

### View Real-time Logs
```bash
fly logs
```

### Check Application Status
```bash
fly status
```

### Open Dashboard
```bash
fly dashboard
```

### SSH into Container
```bash
fly ssh console
```

## Scaling (if needed)

### Increase Memory/CPU
```bash
# Scale to 1GB RAM
fly scale memory 1024

# Scale to 2 CPUs
fly scale vm shared-cpu-2x
```

### Add More Instances (Multiple Regions)
```bash
# Add instance in EU
fly scale count 2 --region fra

# Check instances
fly scale show
```

## Troubleshooting

### Build Fails
```bash
# Check build logs
fly logs --tail

# Try local Docker build first
docker build -t riptide-test .
docker run -p 8080:8080 riptide-test
```

### Can't Connect
```bash
# Check if app is running
fly status

# Check logs for errors
fly logs

# Verify ports are exposed
cat fly.toml | grep -A 5 services
```

### Connection Refused
- Make sure your upstream provider allows connections from Fly.io IPs
- Check if control plane API is accessible from Fly.io
- Verify firewall rules on your control plane

## Using Mock API (For Testing)

If you don't have a control plane API yet:

### Option 1: Deploy Mock API Separately

```bash
# Create a separate app for mock API
fly launch --name riptide-mock-api --no-deploy

# Deploy with Python image
fly deploy --config fly.mock.toml
```

### Option 2: Use Fly.io Internal Network

Deploy both proxy and mock API, connect them via internal network.

## Cost Estimate

### Free Tier (Fly.io)
- 3 shared-cpu-1x VMs with 256MB RAM
- 160GB outbound data transfer
- **Perfect for demo!**

### Paid (if you exceed free tier)
- ~$2/month for 512MB VM
- $0.02/GB for bandwidth

## Cleanup (After Demo)

```bash
# Destroy the app
fly apps destroy riptide-demo

# Or just suspend it
fly scale count 0
```

## Custom Domain (Optional)

```bash
# Add your domain
fly certs add demo.yourdomain.com

# Get DNS records
fly certs show demo.yourdomain.com
```

## Security Tips

1. **Don't commit config.json** with real credentials
2. **Use Fly.io secrets** for sensitive data
3. **Restrict access** by IP if possible
4. **Monitor logs** for suspicious activity
5. **Set resource limits** to prevent abuse

## Support

### Fly.io Documentation
- https://fly.io/docs/

### Fly.io Community
- https://community.fly.io/

### Riptide Proxy Issues
- Check logs: `fly logs`
- Check config: `fly ssh console` → `cat /etc/riptide/config.json`

## Quick Reference

```bash
# Deploy
fly deploy

# View logs
fly logs

# Check status
fly status

# Open dashboard
fly dashboard

# SSH into app
fly ssh console

# Scale resources
fly scale memory 1024

# Stop app
fly scale count 0

# Start app
fly scale count 1

# Destroy app
fly apps destroy riptide-demo
```

---

**🎉 Your Riptide proxy is now live and ready for client demo!**

Share this URL with your client:
```
http://testuser:testpass@riptide-demo.fly.dev:8080
```

(Replace with your actual app name and credentials)

