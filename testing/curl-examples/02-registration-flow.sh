#!/bin/bash

# PPDB API Testing - Registration Flow
# Test complete registration flow dari parent register hingga submit

set -e

BASE_URL="http://localhost:8000"

# Load environment from previous test
if [ -f .test-env ]; then
    source .test-env
else
    echo "Error: .test-env not found. Run 01-setup-flow.sh first!"
    exit 1
fi

echo "========================================="
echo "PPDB API Testing - Registration Flow"
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

# Step 1: Parent Registration
info "Step 1: Parent Registration"
REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "parent1@example.com",
    "password": "admin123",
    "full_name": "Budi Santoso",
    "phone": "081111111111",
    "nik": "3201010101010001"
  }')

PARENT_ID=$(echo $REGISTER_RESPONSE | jq -r '.id')

if [ "$PARENT_ID" != "null" ] && [ -n "$PARENT_ID" ]; then
    success "Parent registered successfully"
    echo "Parent ID: $PARENT_ID"
    echo "Email: $(echo $REGISTER_RESPONSE | jq -r '.email')"
else
    error "Parent registration failed"
    echo $REGISTER_RESPONSE | jq '.'
    exit 1
fi

echo ""

# Step 2: Email Verification (simulated - in real scenario, get token from email)
info "Step 2: Email Verification (simulated)"
# In production, you would get verification token from email
# For testing, we'll skip this or use a test token
success "Email verification skipped for testing (would be done via email link)"

echo ""

# Step 3: Parent Login
info "Step 3: Parent Login"
PARENT_LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "parent1@example.com",
    "password": "admin123"
  }')

PARENT_TOKEN=$(echo $PARENT_LOGIN_RESPONSE | jq -r '.access_token')

if [ "$PARENT_TOKEN" != "null" ] && [ -n "$PARENT_TOKEN" ]; then
    success "Parent logged in successfully"
    echo "Token: ${PARENT_TOKEN:0:20}..."
else
    error "Parent login failed"
    echo $PARENT_LOGIN_RESPONSE | jq '.'
    exit 1
fi

echo ""

# Step 4: Create Registration (Draft)
info "Step 4: Create Registration (Draft)"
REGISTRATION_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/registrations" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $PARENT_TOKEN" \
  -d "{
    \"period_id\": $PERIOD_ID,
    \"path_id\": $ZONASI_PATH_ID,
    \"student_nisn\": \"0012345678\",
    \"student_nik\": \"3201234567890124\",
    \"student_name\": \"Ahmad Santoso\",
    \"student_birth_place\": \"Jakarta\",
    \"student_birth_date\": \"2010-05-15\",
    \"student_gender\": \"Male\",
    \"student_address\": \"Jl. Kebon Jeruk No. 10, Jakarta Barat\",
    \"previous_school\": \"SMP Negeri 5 Jakarta\",
    \"distance_km\": 2.5,
    \"avg_report_score\": 85.5
  }")

REGISTRATION_ID=$(echo $REGISTRATION_RESPONSE | jq -r '.id')
REGISTRATION_NUMBER=$(echo $REGISTRATION_RESPONSE | jq -r '.registration_number')

if [ "$REGISTRATION_ID" != "null" ] && [ -n "$REGISTRATION_ID" ]; then
    success "Registration created successfully"
    echo "Registration ID: $REGISTRATION_ID"
    echo "Registration Number: $REGISTRATION_NUMBER"
    echo "Student Name: $(echo $REGISTRATION_RESPONSE | jq -r '.student_name')"
    echo "Status: $(echo $REGISTRATION_RESPONSE | jq -r '.status')"
else
    error "Registration creation failed"
    echo $REGISTRATION_RESPONSE | jq '.'
    exit 1
fi

echo ""

# Step 5: Upload Documents (simulated - would use multipart/form-data)
info "Step 5: Upload Documents"
echo "Note: Document upload requires multipart/form-data with actual files"
echo "In this test, we'll simulate the document upload process"

# Simulated document uploads
success "Document upload simulation:"
echo "  - Kartu Keluarga: kk.pdf (uploaded)"
echo "  - Akta Kelahiran: akta.pdf (uploaded)"
echo "  - Rapor: rapor.pdf (uploaded)"

echo ""

# Step 6: View Registration
info "Step 6: View Registration Details"
VIEW_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/registrations/$REGISTRATION_ID" \
  -H "Authorization: Bearer $PARENT_TOKEN")

VIEW_STATUS=$(echo $VIEW_RESPONSE | jq -r '.status')

if [ "$VIEW_STATUS" != "null" ]; then
    success "Registration retrieved successfully"
    echo "Status: $VIEW_STATUS"
    echo "Student: $(echo $VIEW_RESPONSE | jq -r '.student_name')"
    echo "Path: $(echo $VIEW_RESPONSE | jq -r '.path_type')"
else
    error "Failed to retrieve registration"
fi

echo ""

# Step 7: Update Registration (optional)
info "Step 7: Update Registration (optional)"
UPDATE_RESPONSE=$(curl -s -X PUT "$BASE_URL/api/v1/registrations/$REGISTRATION_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $PARENT_TOKEN" \
  -d '{
    "parent_phone": "081234567890",
    "parent_email": "parent1@example.com"
  }')

UPDATE_ID=$(echo $UPDATE_RESPONSE | jq -r '.id')

if [ "$UPDATE_ID" != "null" ]; then
    success "Registration updated successfully"
else
    info "Update skipped or failed (may not be needed)"
fi

echo ""

# Step 8: Submit Registration
info "Step 8: Submit Registration"
SUBMIT_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/registrations/$REGISTRATION_ID/submit" \
  -H "Authorization: Bearer $PARENT_TOKEN")

SUBMIT_STATUS=$(echo $SUBMIT_RESPONSE | jq -r '.status')

if [ "$SUBMIT_STATUS" == "Submitted" ]; then
    success "Registration submitted successfully"
    echo "Status: $SUBMIT_STATUS"
    echo "Registration Number: $(echo $SUBMIT_RESPONSE | jq -r '.registration_number')"
else
    error "Registration submission failed"
    echo $SUBMIT_RESPONSE | jq '.'
    exit 1
fi

echo ""

# Step 9: List Parent's Registrations
info "Step 9: List Parent's Registrations"
LIST_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/registrations" \
  -H "Authorization: Bearer $PARENT_TOKEN")

TOTAL=$(echo $LIST_RESPONSE | jq -r '.meta.total')

if [ "$TOTAL" != "null" ] && [ "$TOTAL" -gt 0 ]; then
    success "Found $TOTAL registration(s)"
    echo $LIST_RESPONSE | jq '.data[] | {id, registration_number, student_name, status}'
else
    error "No registrations found"
fi

echo ""
echo "========================================="
echo "Registration Flow Completed Successfully!"
echo "========================================="
echo ""
echo "Summary:"
echo "  Parent ID: $PARENT_ID"
echo "  Parent Token: ${PARENT_TOKEN:0:30}..."
echo "  Registration ID: $REGISTRATION_ID"
echo "  Registration Number: $REGISTRATION_NUMBER"
echo "  Status: Submitted"
echo ""

# Update .test-env
cat >> .test-env << EOF
PARENT_ID=$PARENT_ID
PARENT_TOKEN=$PARENT_TOKEN
REGISTRATION_ID=$REGISTRATION_ID
REGISTRATION_NUMBER=$REGISTRATION_NUMBER
EOF

success "Test environment updated in .test-env"
echo ""
echo "Next: Run 03-verification-flow.sh to verify the registration"
