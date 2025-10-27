#!/bin/bash
# Test script for the Animal Shelter Donation Thermometer API

set -e

BASE_URL="${BASE_URL:-http://localhost:8080}"
EDIT_KEY="${THERMOMETER_EDIT_KEY:-test-key-123}"

echo "Testing Animal Shelter Donation Thermometer API"
echo "Base URL: $BASE_URL"
echo "Edit Key: $EDIT_KEY"
echo ""

echo "1. Testing health check..."
curl -s "$BASE_URL/health"
echo -e "\n"

echo "2. Testing root endpoint..."
curl -s "$BASE_URL/"
echo ""

echo "3. Getting current config..."
curl -s "$BASE_URL/config" | jq .
echo ""

echo "4. Uploading sample CSV..."
curl -s -X POST "$BASE_URL/admin/upload" \
  -H "Authorization: Bearer $EDIT_KEY" \
  -F "file=@sample-teams.csv" | jq .
echo ""

echo "5. Getting updated config..."
curl -s "$BASE_URL/config" | jq .
echo ""

echo "6. Testing thermometer image endpoint..."
curl -s "$BASE_URL/thermometer.png" -o /tmp/thermometer.png
echo "Image saved to /tmp/thermometer.png"
file /tmp/thermometer.png
echo ""

echo "7. Testing auth failure (wrong key)..."
curl -s -X POST "$BASE_URL/admin/upload" \
  -H "Authorization: Bearer wrong-key" \
  -F "file=@sample-teams.csv" || echo "Expected failure"
echo ""

echo "8. Updating config via JSON..."
curl -s -X POST "$BASE_URL/admin/config" \
  -H "Authorization: Bearer $EDIT_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "2025 Spring Fundraiser",
    "goal": 25000.0,
    "teams": [
      {
        "name": "Test Team",
        "image_url": null,
        "total_raised": 5000.0
      }
    ]
  }' | jq .
echo ""

echo "All tests completed!"
