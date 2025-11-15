#!/bin/bash

# PPDB API Testing - Simple Setup Flow
# Using existing seed data users

set -e

BASE_URL="http://localhost:8000"

echo "========================================="
echo "PPDB API Testing - Simple Setup Flow"
echo "========================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

success() { echo -e "${GREEN}✓ $1${NC}"; }
error() { echo -e "${RED}✗ $1${NC}"; }
info() { echo -e "${YELLOW}→ $1${NC}"; }

# Step 1: SuperAdmin Login
info "Step 1: SuperAdmin Login"
SUPERADMIN_TOKEN=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"superadmin@ppdb.com","password":"admin123"}' \
  | jq -r '.access_token')

if [ "$SUPERADMIN_TOKEN" != "null" ] && [ -n "$SUPERADMIN_TOKEN" ]; then
    success "SuperAdmin logged in"
else
    error "SuperAdmin login failed"
    exit 1
fi

# Step 2: School Admin Login (using existing seed data)
info "Step 2: School Admin Login (SMA Negeri 1 Jakarta)"
ADMIN_TOKEN=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@sman1jkt.sch.id","password":"admin123"}' \
  | jq -r '.access_token')

if [ "$ADMIN_TOKEN" != "null" ] && [ -n "$ADMIN_TOKEN" ]; then
    success "School Admin logged in"
    SCHOOL_ID=1
else
    error "Admin login failed"
    exit 1
fi

# Step 3: List existing periods
info "Step 3: List Existing Periods"
PERIODS=$(curl -s -X GET "$BASE_URL/api/v1/periods" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

PERIOD_COUNT=$(echo $PERIODS | jq '.data | length')
success "Found $PERIOD_COUNT period(s)"

if [ "$PERIOD_COUNT" -gt 0 ]; then
    PERIOD_ID=$(echo $PERIODS | jq -r '.data[0].id')
    PERIOD_NAME=$(echo $PERIODS | jq -r '.data[0].name')
    success "Using period: $PERIOD_NAME (ID: $PERIOD_ID)"
else
    info "No periods found, creating new one..."
    
    # Create new period
    PERIOD_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/periods" \
      -H "Content-Type: application/json" \
      -H "Authorization: Bearer $ADMIN_TOKEN" \
      -d '{
        "name": "PPDB Test 2025/2026",
        "academic_year": "2025/2026",
        "level": "SMA",
        "start_date": "2025-06-01T00:00:00Z",
        "end_date": "2026-06-30T23:59:59Z",
        "registration_start": "2025-01-01T00:00:00Z",
        "registration_end": "2025-12-31T23:59:59Z",
        "announcement_date": "2026-01-15T00:00:00Z",
        "reenrollment_deadline": "2026-02-01T00:00:00Z"
      }')
    
    PERIOD_ID=$(echo $PERIOD_RESPONSE | jq -r '.id')
    if [ "$PERIOD_ID" != "null" ]; then
        success "Period created (ID: $PERIOD_ID)"
    else
        error "Period creation failed"
        echo $PERIOD_RESPONSE | jq '.'
        exit 1
    fi
fi

# Step 4: List paths for the period
info "Step 4: List Registration Paths"
PATHS=$(curl -s -X GET "$BASE_URL/api/v1/periods/$PERIOD_ID/paths" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

PATH_COUNT=$(echo $PATHS | jq '. | length')
success "Found $PATH_COUNT path(s)"

if [ "$PATH_COUNT" -gt 0 ]; then
    ZONASI_PATH_ID=$(echo $PATHS | jq -r '.[] | select(.path_type=="zonasi") | .id')
    success "Zonasi Path ID: $ZONASI_PATH_ID"
else
    info "No paths found, you may need to create them"
fi

# Step 5: Parent Login
info "Step 5: Parent Login"
PARENT_TOKEN=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"parent1@example.com","password":"admin123"}' \
  | jq -r '.access_token')

if [ "$PARENT_TOKEN" != "null" ] && [ -n "$PARENT_TOKEN" ]; then
    success "Parent logged in"
else
    error "Parent login failed"
    exit 1
fi

echo ""
echo "========================================="
echo "Setup Complete!"
echo "========================================="
echo ""
echo "Summary:"
echo "  SuperAdmin Token: ${SUPERADMIN_TOKEN:0:30}..."
echo "  Admin Token: ${ADMIN_TOKEN:0:30}..."
echo "  Parent Token: ${PARENT_TOKEN:0:30}..."
echo "  School ID: $SCHOOL_ID"
echo "  Period ID: $PERIOD_ID"
if [ -n "$ZONASI_PATH_ID" ]; then
    echo "  Zonasi Path ID: $ZONASI_PATH_ID"
fi
echo ""

# Save to file
cat > .test-env << EOF
SCHOOL_ID=$SCHOOL_ID
PERIOD_ID=$PERIOD_ID
ZONASI_PATH_ID=$ZONASI_PATH_ID
SUPERADMIN_TOKEN=$SUPERADMIN_TOKEN
ADMIN_TOKEN=$ADMIN_TOKEN
PARENT_TOKEN=$PARENT_TOKEN
EOF

success "Environment saved to .test-env"
echo ""
echo "Next: Run registration test"
echo "  bash 02-registration-flow-simple.sh"
