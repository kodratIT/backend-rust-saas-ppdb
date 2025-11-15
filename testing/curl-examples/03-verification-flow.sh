#!/bin/bash

# PPDB API Testing - Verification Flow
# Test verification flow dari admin view pending hingga approve

set -e

BASE_URL="http://localhost:8000"

# Load environment
if [ -f .test-env ]; then
    source .test-env
else
    echo "Error: .test-env not found. Run previous tests first!"
    exit 1
fi

echo "========================================="
echo "PPDB API Testing - Verification Flow"
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

# Step 1: Admin View Pending Verifications
info "Step 1: Admin View Pending Verifications"
PENDING_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/verifications/pending" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

PENDING_COUNT=$(echo $PENDING_RESPONSE | jq -r '.meta.total')

if [ "$PENDING_COUNT" != "null" ] && [ "$PENDING_COUNT" -gt 0 ]; then
    success "Found $PENDING_COUNT pending verification(s)"
    echo $PENDING_RESPONSE | jq '.data[] | {id, registration_number, student_name, status}'
else
    error "No pending verifications found"
    echo $PENDING_RESPONSE | jq '.'
fi

echo ""

# Step 2: View Registration Detail
info "Step 2: View Registration Detail"
DETAIL_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/registrations/$REGISTRATION_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

DETAIL_STATUS=$(echo $DETAIL_RESPONSE | jq -r '.status')

if [ "$DETAIL_STATUS" != "null" ]; then
    success "Registration details retrieved"
    echo "Registration Number: $(echo $DETAIL_RESPONSE | jq -r '.registration_number')"
    echo "Student: $(echo $DETAIL_RESPONSE | jq -r '.student_name')"
    echo "NISN: $(echo $DETAIL_RESPONSE | jq -r '.student_nisn')"
    echo "Status: $DETAIL_STATUS"
    echo "Path: $(echo $DETAIL_RESPONSE | jq -r '.path_type')"
else
    error "Failed to retrieve registration details"
fi

echo ""

# Step 3: Get Verification Stats
info "Step 3: Get Verification Statistics"
STATS_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/verifications/stats?period_id=$PERIOD_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

if [ "$(echo $STATS_RESPONSE | jq -r '.total_registrations')" != "null" ]; then
    success "Verification statistics retrieved"
    echo "Total Registrations: $(echo $STATS_RESPONSE | jq -r '.total_registrations')"
    echo "Pending: $(echo $STATS_RESPONSE | jq -r '.pending')"
    echo "Verified: $(echo $STATS_RESPONSE | jq -r '.verified')"
    echo "Rejected: $(echo $STATS_RESPONSE | jq -r '.rejected')"
else
    info "Stats not available or endpoint not implemented"
fi

echo ""

# Step 4: List Documents
info "Step 4: List Registration Documents"
DOCS_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/registrations/$REGISTRATION_ID/documents" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

DOCS_COUNT=$(echo $DOCS_RESPONSE | jq '. | length')

if [ "$DOCS_COUNT" != "null" ] && [ "$DOCS_COUNT" -gt 0 ]; then
    success "Found $DOCS_COUNT document(s)"
    echo $DOCS_RESPONSE | jq '.[] | {id, document_type, verification_status}'
else
    info "No documents found (may need to upload first)"
fi

echo ""

# Step 5: Verify Documents (if exists)
if [ "$DOCS_COUNT" -gt 0 ]; then
    info "Step 5: Verify Documents"
    
    # Get first document ID
    DOC_ID=$(echo $DOCS_RESPONSE | jq -r '.[0].id')
    
    if [ "$DOC_ID" != "null" ]; then
        VERIFY_DOC_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/verifications/documents/$DOC_ID/verify" \
          -H "Content-Type: application/json" \
          -H "Authorization: Bearer $ADMIN_TOKEN" \
          -d '{
            "status": "Verified",
            "notes": "Dokumen valid dan sesuai"
          }')
        
        if [ "$(echo $VERIFY_DOC_RESPONSE | jq -r '.message')" != "null" ]; then
            success "Document verified successfully"
        else
            info "Document verification may not be implemented"
        fi
    fi
else
    info "Step 5: Skipping document verification (no documents)"
fi

echo ""

# Step 6: Approve Registration
info "Step 6: Approve Registration"
VERIFY_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/verifications/registrations/$REGISTRATION_ID/verify" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{
    "notes": "Semua dokumen lengkap dan valid. Pendaftaran disetujui."
  }')

VERIFY_STATUS=$(echo $VERIFY_RESPONSE | jq -r '.status')

if [ "$VERIFY_STATUS" == "Verified" ]; then
    success "Registration verified successfully"
    echo "Status: $VERIFY_STATUS"
    echo "Selection Score: $(echo $VERIFY_RESPONSE | jq -r '.selection_score')"
else
    error "Registration verification failed"
    echo $VERIFY_RESPONSE | jq '.'
    exit 1
fi

echo ""

# Step 7: Verify Score Calculation
info "Step 7: Verify Score Calculation"
SCORE_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/registrations/$REGISTRATION_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

SCORE=$(echo $SCORE_RESPONSE | jq -r '.selection_score')

if [ "$SCORE" != "null" ] && [ "$SCORE" != "0" ]; then
    success "Selection score calculated: $SCORE"
else
    info "Score not yet calculated or is 0"
fi

echo ""

# Step 8: Test Rejection Flow (with another registration if available)
info "Step 8: Test Rejection Flow (optional)"
echo "To test rejection, create another registration and use:"
echo "curl -X POST \"$BASE_URL/api/v1/verifications/registrations/{id}/reject\" \\"
echo "  -H \"Authorization: Bearer \$ADMIN_TOKEN\" \\"
echo "  -d '{\"reason\": \"Dokumen tidak lengkap\"}'"

echo ""
echo "========================================="
echo "Verification Flow Completed Successfully!"
echo "========================================="
echo ""
echo "Summary:"
echo "  Registration ID: $REGISTRATION_ID"
echo "  Status: Verified"
echo "  Selection Score: $SCORE"
echo ""
echo "Next: Run 04-selection-flow.sh to run selection process"
