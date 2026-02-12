#!/bin/bash
# API Verification Script for Dev Mode Package Installation
# This script verifies that the API endpoints work correctly in dev mode
#
# Prerequisites:
# 1. Backend running with PINAS_DEV_MODE=true
# 2. curl and jq installed
#
# Usage: ./api-verification.sh

set -e

BASE_URL="${PINAS_API_URL:-http://localhost:3388}"

echo "=== Dev Mode API Verification ==="
echo "Base URL: $BASE_URL"
echo ""

# Step 1: Install Docker package
echo "1. POST /api/packages/install - Installing Docker..."
INSTALL_RESPONSE=$(curl -s -X POST "$BASE_URL/api/packages/install" \
  -H "Content-Type: application/json" \
  -d '{"package_id": "docker"}')

echo "   Response: $INSTALL_RESPONSE"

# Extract task_id and package_id
TASK_ID=$(echo "$INSTALL_RESPONSE" | jq -r '.task_id // empty')
PACKAGE_ID=$(echo "$INSTALL_RESPONSE" | jq -r '.package_id // empty')

if [ -z "$TASK_ID" ] || [ -z "$PACKAGE_ID" ]; then
  echo "   FAIL: Response missing task_id or package_id"
  exit 1
fi

echo "   PASS: Got task_id=$TASK_ID, package_id=$PACKAGE_ID"
echo ""

# Step 2: Check task status
echo "2. GET /api/packages/task/$TASK_ID - Checking task status..."
TASK_RESPONSE=$(curl -s "$BASE_URL/api/packages/task/$TASK_ID")

echo "   Response: $TASK_RESPONSE"

TASK_STATUS=$(echo "$TASK_RESPONSE" | jq -r '.status // empty')

if [ "$TASK_STATUS" != "completed" ]; then
  echo "   FAIL: Task status is '$TASK_STATUS', expected 'completed'"
  exit 1
fi

echo "   PASS: Task status is 'completed'"
echo ""

# Step 3: Check package is installed
echo "3. GET /api/packages - Checking installed packages..."
PACKAGES_RESPONSE=$(curl -s "$BASE_URL/api/packages")

echo "   Response: $PACKAGES_RESPONSE"

DOCKER_STATUS=$(echo "$PACKAGES_RESPONSE" | jq -r '.[] | select(.id == "docker") | .status // empty')

if [ "$DOCKER_STATUS" != "installed" ]; then
  echo "   FAIL: Docker status is '$DOCKER_STATUS', expected 'installed'"
  exit 1
fi

echo "   PASS: Docker package is 'installed'"
echo ""

echo "=== All Verifications Passed ==="
echo ""
echo "Summary:"
echo "  - POST /api/packages/install returns task_id and package_id"
echo "  - GET /api/packages/task/:id returns status: completed"
echo "  - GET /api/packages includes docker with status: installed"
