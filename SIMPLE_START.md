# 🚀 Simple Start Guide - Riptide Proxy MVP

## **How to Run This Project (3 Steps)**

---

## ✅ Step 1: Build the Project

Open PowerShell/Command Prompt in this folder and run:

```bash
cargo build --release --no-default-features
```

**Wait for:** "Finished release profile" message (takes ~30 seconds)

**You get:** Two programs in `target/release/`
- `riptide.exe` - The proxy server
- `test-client.exe` - Testing tool

---

## ✅ Step 2: Start the Services

You need **2 terminals open**:

### Terminal 1: Start Mock API
```bash
python mock_api_server.py
```

**You should see:**
```
🚀 Mock Control Plane API Server
📍 Address: http://127.0.0.1:8000

📋 Test Users:
  • testuser:testpass
  • premium:premium123
  • limited:limited123
```

**Leave this running!**

### Terminal 2: Start Proxy
```bash
target\release\riptide.exe --config config.json
```

**You should see:**
```
🎯 Listening on 0.0.0.0:8080
🔄 User sync started
```

**Leave this running!**

---

## ✅ Step 3: Test It Works

### Option A: Web Dashboard (Easiest!)

1. **Open:** `MVP_DASHBOARD.html` (double-click it)
2. **Sign in:** 
   - Username: `testuser`
   - Password: `testpass`
3. **Click test buttons** to verify everything works!

### Option B: Command Line Test

```bash
curl.exe -x http://testuser:testpass@127.0.0.1:8080 http://httpbin.org/ip
```

**Expected result:** Shows an IP address (your upstream provider's IP)

### Option C: Chrome Browser

```bash
# Launch Chrome with proxy
"C:\Program Files\Google\Chrome\Application\chrome.exe" --proxy-server="http://127.0.0.1:8080" --new-window "http://httpbin.org/ip"
```

**When prompted:**
- Username: `testuser`
- Password: `testpass`

**Expected result:** Page shows IP address in JSON format

---

## 🎯 That's It! You're Done!

If you see an IP address, **the proxy is working!** ✅

---

## 📋 Quick Reference

### Test Users (Choose One):

| Username | Password | Max Threads | Speed Limit | Bandwidth |
|----------|----------|-------------|-------------|-----------|
| testuser | testpass | 100 | 50 Mbps | 10 GB |
| premium | premium123 | 500 | 100 Mbps | 100 GB |
| limited | limited123 | 10 | 10 Mbps | 1 GB |

### Important Files:

- `config.json` - Configuration (edit upstream provider here)
- `mock_api_server.py` - Fake control plane API (for testing)
- `target/release/riptide.exe` - The actual proxy server
- `target/release/test-client.exe` - Load testing tool
- `MVP_DASHBOARD.html` - Beautiful web interface

---

## 🧪 How to Test Different Features

### Test 1: Basic Connection
```bash
curl.exe -x http://testuser:testpass@127.0.0.1:8080 http://httpbin.org/ip
```
✅ Should return IP address

### Test 2: Parameter Mapping (Geographic Targeting)
```bash
curl.exe -x http://testuser-country-us-city-newyork:testpass@127.0.0.1:8080 http://ipinfo.io/json
```
✅ Should show US/New York location

### Test 3: Thread Limits (limited user can only do 10 concurrent)
```bash
target\release\test-client.exe --proxy 127.0.0.1:8080 -u limited -P limited123 -c 15 -d 10
```
✅ Should show: 10 connections succeed, 5 rejected

### Test 4: Load Test (500 concurrent connections)
```bash
target\release\test-client.exe --proxy 127.0.0.1:8080 -u premium -P premium123 -c 500 -d 60
```
✅ Should show: 100% success rate, low CPU/RAM

---

## ❌ Troubleshooting

### Problem: "cargo: command not found"
**Solution:** Install Rust from https://rustup.rs/

### Problem: "python: command not found"
**Solution:** Install Python from https://www.python.org/

### Problem: "pip install flask" needed
**Solution:** Run: `pip install flask`

### Problem: Port 8080 already in use
**Solution:** 
```bash
# Check what's using port 8080
netstat -ano | findstr ":8080"

# Kill that process or change config.json to use different port
```

### Problem: "Connection refused"
**Solution:** Make sure both services are running (Terminal 1 and Terminal 2)

### Problem: "ERR_TUNNEL_CONNECTION_FAILED" in browser
**Solution:** This is normal! Your upstream provider needs to be configured in `config.json`

---

## 🎊 Success Checklist

- [ ] Project built successfully (cargo build)
- [ ] Mock API running (Terminal 1)
- [ ] Proxy running (Terminal 2)
- [ ] curl test returns IP address
- [ ] Dashboard login works
- [ ] Test buttons work in dashboard

**All checked?** Your MVP is working! 🎉

---

## 📞 What to Show the Client

1. ✅ **Open MVP_DASHBOARD.html**
2. ✅ **Sign in with testuser/testpass**
3. ✅ **Show all 6 features with green checkmarks**
4. ✅ **Click test buttons** - they all pass!
5. ✅ **Show performance metrics** - 500 connections!
6. ✅ **Run load test** - 100% success rate!

**Client will be impressed!** 🚀

---

**Last Updated:** 2025-11-20  
**Status:** ✅ Ready to Execute  
**Difficulty:** ⭐ Easy (3 steps!)

