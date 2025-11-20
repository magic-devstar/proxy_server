# Quick Deploy with ngrok - Get Public URL Now

## Step 1: Download ngrok

Visit: https://ngrok.com/download
Or use chocolatey: `choco install ngrok`

## Step 2: Sign up (free)

Visit: https://dashboard.ngrok.com/signup
Get your authtoken from: https://dashboard.ngrok.com/get-started/your-authtoken

## Step 3: Setup ngrok

```powershell
# After downloading ngrok.exe to your Downloads folder
cd $env:USERPROFILE\Downloads
.\ngrok.exe config add-authtoken YOUR_AUTH_TOKEN
```

## Step 4: Start Mock API (Terminal 1)

```powershell
cd E:\JOB\rust
python mock_api_server.py
```

## Step 5: Build & Start Proxy (Terminal 2)

```powershell
cd E:\JOB\rust

# Quick build without issues
cargo build --release --no-default-features

# Or if build fails, use debug mode
cargo build

# Run proxy
.\target\release\riptide.exe --config config.json
# OR if release failed:
.\target\debug\riptide.exe --config config.json
```

## Step 6: Create ngrok Tunnel (Terminal 3)

```powershell
cd $env:USERPROFILE\Downloads
.\ngrok.exe tcp 8080
```

You'll see output like:
```
Forwarding    tcp://0.tcp.ngrok.io:12345 -> localhost:8080
```

## Step 7: Share with Client

Your public URL is: **tcp://0.tcp.ngrok.io:12345**

Test command:
```bash
curl -x http://testuser:testpass@0.tcp.ngrok.io:12345 https://httpbin.org/ip
```

## Alternative: HTTP Tunnel (if you need HTTP instead of TCP)

```powershell
.\ngrok.exe http 8080
```

You'll get: `https://something.ngrok.io` - even better for browser testing!

