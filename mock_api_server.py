#!/usr/bin/env python3
"""
Mock Control Plane API Server for Testing Riptide Proxy

This server simulates the control plane API endpoints:
- GET /api/riptide?node=X - Returns test users
- GET /api/blacklists?node=X - Returns empty blacklist
- POST /api/riptide/report - Accepts statistics

Usage:
    python3 mock_api_server.py

The server will start on http://127.0.0.1:8000
"""

from flask import Flask, request, jsonify
import json
from datetime import datetime

app = Flask(__name__)

# Mock users database
USERS = [
    {
        "username": "testuser",
        "password": "testpass",
        "user_id": 1001,
        "user_type": "residential",
        "plan": {
            "id": 5001,
            "status": "active",
            "max_threads": 100,
            "max_throughput": 50,  # 50 Mbps
            "max_bytes": 10737418240,  # 10 GB
            "bytes_used": 0
        }
    },
    {
        "username": "premium",
        "password": "premium123",
        "user_id": 1002,
        "user_type": "residential",
        "plan": {
            "id": 5002,
            "status": "active",
            "max_threads": 500,
            "max_throughput": 100,  # 100 Mbps
            "max_bytes": 107374182400,  # 100 GB
            "bytes_used": 0
        }
    },
    {
        "username": "limited",
        "password": "limited123",
        "user_id": 1003,
        "user_type": "residential",
        "plan": {
            "id": 5003,
            "status": "active",
            "max_threads": 10,
            "max_throughput": 10,  # 10 Mbps
            "max_bytes": 1073741824,  # 1 GB
            "bytes_used": 0
        }
    }
]

@app.route('/api/riptide', methods=['GET'])
def get_users():
    """Return users and plans"""
    node = request.args.get('node', 'default')
    api_key = request.headers.get('api-key', '')
    
    print(f"[{datetime.now()}] GET /api/riptide?node={node} | API-Key: {api_key}")
    
    # Return all users
    return jsonify(USERS)

@app.route('/api/blacklists', methods=['GET'])
def get_blacklists():
    """Return blacklisted domains/ports (empty for MVP)"""
    node = request.args.get('node', 'default')
    api_key = request.headers.get('api-key', '')
    
    print(f"[{datetime.now()}] GET /api/blacklists?node={node} | API-Key: {api_key}")
    
    # Return empty blacklist for MVP
    return jsonify([])

@app.route('/api/riptide/report', methods=['POST'])
def report_stats():
    """Accept statistics reports"""
    api_key = request.headers.get('api-key', '')
    data = request.get_json()
    
    print(f"[{datetime.now()}] POST /api/riptide/report | API-Key: {api_key}")
    print(f"  Reports: {len(data)} entries")
    
    for report in data:
        print(f"    - {report.get('key')}: "
              f"{report.get('traffic')} bytes, "
              f"{report.get('current_threads')} threads, "
              f"{report.get('current_throughput'):.2f} MB/s")
        
        # Update bytes_used for testing
        key = report.get('key', '')
        traffic = report.get('traffic', 0)
        
        # Find user and update bytes_used
        for user in USERS:
            user_key = f"{user['user_type']}:{user['username']}:{user['plan']['id']}:{user['user_id']}"
            if key == user_key:
                user['plan']['bytes_used'] += traffic
                print(f"      Updated {user['username']} bytes_used: {user['plan']['bytes_used']}")
    
    return jsonify({"status": "ok"})

@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint"""
    return jsonify({"status": "healthy", "timestamp": datetime.now().isoformat()})

if __name__ == '__main__':
    print("🚀 Mock Control Plane API Server")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print("📍 Address: http://127.0.0.1:8000")
    print("")
    print("📋 Test Users:")
    for user in USERS:
        print(f"  • {user['username']}:{user['password']}")
        print(f"    - Threads: {user['plan']['max_threads']}")
        print(f"    - Speed: {user['plan']['max_throughput']} Mbps")
        print(f"    - Bandwidth: {user['plan']['max_bytes'] / (1024**3):.1f} GB")
        print("")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print("")
    
    app.run(host='127.0.0.1', port=8000, debug=True)

