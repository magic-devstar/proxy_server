#!/bin/bash

# Riptide Proxy Test Suite

set -e

echo "🧪 Riptide Proxy Test Suite"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
PROXY_HOST="${PROXY_HOST:-127.0.0.1}"
PROXY_PORT="${PROXY_PORT:-8080}"
USERNAME="${TEST_USER:-testuser}"
PASSWORD="${TEST_PASS:-testpass}"
TARGET="${TEST_TARGET:-httpbin.org:80}"

echo "📍 Proxy: $PROXY_HOST:$PROXY_PORT"
echo "👤 User: $USERNAME"
echo "🎯 Target: $TARGET"
echo ""

# Check if binaries exist
if [ ! -f "target/release/test-client" ]; then
    echo -e "${RED}❌ test-client binary not found. Run ./build.sh first.${NC}"
    exit 1
fi

# Test 1: Single Connection
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 1: Single Connection Test"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
./target/release/test-client \
    --proxy "$PROXY_HOST:$PROXY_PORT" \
    --username "$USERNAME" \
    --password "$PASSWORD" \
    --target "$TARGET" \
    --connections 1 \
    --duration 5

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Test 1 PASSED${NC}"
else
    echo -e "${RED}❌ Test 1 FAILED${NC}"
    exit 1
fi

echo ""
sleep 2

# Test 2: Multiple Connections
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 2: Multiple Connections (10)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
./target/release/test-client \
    --proxy "$PROXY_HOST:$PROXY_PORT" \
    --username "$USERNAME" \
    --password "$PASSWORD" \
    --target "$TARGET" \
    --connections 10 \
    --duration 5

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Test 2 PASSED${NC}"
else
    echo -e "${RED}❌ Test 2 FAILED${NC}"
    exit 1
fi

echo ""
sleep 2

# Test 3: SOCKS5
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 3: SOCKS5 Protocol"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
./target/release/test-client \
    --proxy "$PROXY_HOST:$PROXY_PORT" \
    --username "$USERNAME" \
    --password "$PASSWORD" \
    --target "$TARGET" \
    --socks5 \
    --connections 5 \
    --duration 5

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Test 3 PASSED${NC}"
else
    echo -e "${RED}❌ Test 3 FAILED${NC}"
    exit 1
fi

echo ""
sleep 2

# Test 4: Parameter Mapping
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 4: Parameter Mapping (country-us-session-test)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
./target/release/test-client \
    --proxy "$PROXY_HOST:$PROXY_PORT" \
    --username "$USERNAME-country-us-session-test123" \
    --password "$PASSWORD" \
    --target "$TARGET" \
    --connections 3 \
    --duration 5

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Test 4 PASSED${NC}"
else
    echo -e "${YELLOW}⚠️  Test 4 FAILED (expected if user sync not configured)${NC}"
fi

echo ""
sleep 2

# Test 5: Load Test
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 5: Load Test (50 connections, 10s)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
./target/release/test-client \
    --proxy "$PROXY_HOST:$PROXY_PORT" \
    --username "$USERNAME" \
    --password "$PASSWORD" \
    --target "$TARGET" \
    --connections 50 \
    --duration 10

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Test 5 PASSED${NC}"
else
    echo -e "${RED}❌ Test 5 FAILED${NC}"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ All tests completed!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

