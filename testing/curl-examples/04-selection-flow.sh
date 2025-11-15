#!/bin/bash

# PPDB API Testing - Selection & Announcement Flow
# Test selection process dari calculate scores hingga announcement

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
echo "PPDB API Testing - Selection Flow"
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

# Step 1: Calculate Scores
info "Step 1: Calculate Selection Scores"
CALC_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/selection/calculate-scores" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d "{
    \"period_id\": $PERIOD_ID
  }")

CALC_COUNT=$(echo $CALC_RESPONSE | jq -r '.calculated_count')

if [ "$CALC_COUNT" != "null" ]; then
    success "Scores calculated for $CALC_COUNT registration(s)"
    echo "Period ID: $(echo $CALC_RESPONSE | jq -r '.period_id')"
else
    error "Score calculation failed"
    echo $CALC_RESPONSE | jq '.'
fi

echo ""

# Step 2: Update Rankings
info "Step 2: Update Rankings"
RANK_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/selection/update-rankings" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d "{
    \"period_id\": $PERIOD_ID
  }")

RANK_COUNT=$(echo $RANK_RESPONSE | jq -r '.updated_count')

if [ "$RANK_COUNT" != "null" ]; then
    success "Rankings updated for $RANK_COUNT registration(s)"
else
    error "Ranking update failed"
    echo $RANK_RESPONSE | jq '.'
fi

echo ""

# Step 3: View Rankings by Path
info "Step 3: View Rankings - Jalur Zonasi"
ZONASI_RANK_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/selection/rankings?period_id=$PERIOD_ID&path_id=$ZONASI_PATH_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

ZONASI_TOTAL=$(echo $ZONASI_RANK_RESPONSE | jq -r '.meta.total')

if [ "$ZONASI_TOTAL" != "null" ]; then
    success "Zonasi rankings retrieved: $ZONASI_TOTAL student(s)"
    echo "Top 5 Rankings:"
    echo $ZONASI_RANK_RESPONSE | jq '.data[:5] | .[] | {rank, student_name, score, status}'
else
    info "No rankings found for Zonasi path"
fi

echo ""

# Step 4: Get Ranking Statistics
info "Step 4: Get Ranking Statistics"
STATS_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/selection/rankings/stats?period_id=$PERIOD_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

if [ "$(echo $STATS_RESPONSE | jq -r '.period_id')" != "null" ]; then
    success "Ranking statistics retrieved"
    echo $STATS_RESPONSE | jq '.paths[] | {path_name, total_applicants, quota, avg_score}'
else
    info "Statistics not available"
fi

echo ""

# Step 5: Run Selection
info "Step 5: Run Selection Process"
SELECTION_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/announcements/run-selection" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d "{
    \"period_id\": $PERIOD_ID
  }")

TOTAL_ACCEPTED=$(echo $SELECTION_RESPONSE | jq -r '.total_accepted')
TOTAL_REJECTED=$(echo $SELECTION_RESPONSE | jq -r '.total_rejected')

if [ "$TOTAL_ACCEPTED" != "null" ]; then
    success "Selection completed successfully"
    echo "Total Accepted: $TOTAL_ACCEPTED"
    echo "Total Rejected: $TOTAL_REJECTED"
    echo ""
    echo "Results by Path:"
    echo $SELECTION_RESPONSE | jq '.paths[] | {path_name, accepted, rejected, quota}'
else
    error "Selection process failed"
    echo $SELECTION_RESPONSE | jq '.'
    exit 1
fi

echo ""

# Step 6: Get Selection Summary
info "Step 6: Get Selection Summary"
SUMMARY_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/announcements/summary?period_id=$PERIOD_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

if [ "$(echo $SUMMARY_RESPONSE | jq -r '.period_id')" != "null" ]; then
    success "Selection summary retrieved"
    echo "Period: $(echo $SUMMARY_RESPONSE | jq -r '.period_name')"
    echo "Total Registrations: $(echo $SUMMARY_RESPONSE | jq -r '.total_registrations')"
    echo "Total Accepted: $(echo $SUMMARY_RESPONSE | jq -r '.total_accepted')"
    echo "Total Rejected: $(echo $SUMMARY_RESPONSE | jq -r '.total_rejected')"
else
    info "Summary not available"
fi

echo ""

# Step 7: Announce Results
info "Step 7: Announce Results to Students"
ANNOUNCE_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/announcements/announce" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d "{
    \"period_id\": $PERIOD_ID
  }")

ANNOUNCED_COUNT=$(echo $ANNOUNCE_RESPONSE | jq -r '.announced_count')

if [ "$ANNOUNCED_COUNT" != "null" ]; then
    success "Results announced to $ANNOUNCED_COUNT student(s)"
    echo "Emails sent: $(echo $ANNOUNCE_RESPONSE | jq -r '.emails_sent')"
else
    error "Announcement failed"
    echo $ANNOUNCE_RESPONSE | jq '.'
fi

echo ""

# Step 8: Parent Check Result (Public Endpoint)
info "Step 8: Check Result (Public)"
CHECK_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/announcements/check-result?registration_number=$REGISTRATION_NUMBER&nisn=0012345678")

RESULT_STATUS=$(echo $CHECK_RESPONSE | jq -r '.status')

if [ "$RESULT_STATUS" != "null" ]; then
    success "Result check successful"
    echo "Registration Number: $(echo $CHECK_RESPONSE | jq -r '.registration_number')"
    echo "Student Name: $(echo $CHECK_RESPONSE | jq -r '.student_name')"
    echo "Status: $RESULT_STATUS"
    echo "Path: $(echo $CHECK_RESPONSE | jq -r '.path_name')"
    
    if [ "$RESULT_STATUS" == "Accepted" ]; then
        echo "Rank: $(echo $CHECK_RESPONSE | jq -r '.rank')"
        echo "Score: $(echo $CHECK_RESPONSE | jq -r '.score')"
        echo ""
        echo "Payment Information:"
        echo "  Amount: Rp $(echo $CHECK_RESPONSE | jq -r '.payment_amount')"
        echo "  Deadline: $(echo $CHECK_RESPONSE | jq -r '.payment_deadline')"
    else
        echo "Reason: $(echo $CHECK_RESPONSE | jq -r '.rejection_reason')"
    fi
else
    error "Result check failed"
    echo $CHECK_RESPONSE | jq '.'
fi

echo ""

# Step 9: Parent View Own Result (Authenticated)
info "Step 9: Parent View Own Registration Result"
PARENT_RESULT=$(curl -s -X GET "$BASE_URL/api/registrations/$REGISTRATION_ID" \
  -H "Authorization: Bearer $PARENT_TOKEN")

PARENT_STATUS=$(echo $PARENT_RESULT | jq -r '.status')

if [ "$PARENT_STATUS" != "null" ]; then
    success "Parent can view result"
    echo "Status: $PARENT_STATUS"
    
    if [ "$PARENT_STATUS" == "Accepted" ]; then
        echo "Next step: Payment & Enrollment"
    fi
else
    error "Failed to retrieve result"
fi

echo ""
echo "========================================="
echo "Selection Flow Completed Successfully!"
echo "========================================="
echo ""
echo "Summary:"
echo "  Period ID: $PERIOD_ID"
echo "  Total Accepted: $TOTAL_ACCEPTED"
echo "  Total Rejected: $TOTAL_REJECTED"
echo "  Registration Status: $RESULT_STATUS"
echo ""

if [ "$RESULT_STATUS" == "Accepted" ]; then
    echo "✓ Student ACCEPTED - Ready for payment & enrollment"
    echo "  Next: Payment process (if implemented)"
else
    echo "✗ Student REJECTED"
    echo "  Reason: Outside quota or insufficient score"
fi

echo ""
echo "Test completed! Check Swagger UI for more details:"
echo "  http://localhost:8000/swagger-ui/"
