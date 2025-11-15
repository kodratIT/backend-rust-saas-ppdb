#!/bin/bash

# PPDB API - Run All Tests
# Execute complete end-to-end flow test

set -e

BASE_URL="http://localhost:8000"

echo "=============================================="
echo "PPDB API - Complete Test Suite"
echo "=============================================="
echo ""
echo "This will run complete end-to-end flow test covering:"
echo "  ✓ Req 1: Multi-Tenant & RBAC"
echo "  ✓ Req 2: Authentication & Period Management"
echo "  ✓ Req 3: Student Registration"
echo "  ✓ Req 4: Document Verification"
echo "  ✓ Req 5: Score Calculation"
echo "  ✓ Req 6: Selection & Ranking"
echo "  ✓ Req 7: Result Announcement"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

success() { echo -e "${GREEN}✓ $1${NC}"; }
error() { echo -e "${RED}✗ $1${NC}"; }
info() { echo -e "${YELLOW}→ $1${NC}"; }

# Check if backend is running
info "Checking if backend is running..."
if curl -s "$BASE_URL/health" > /dev/null 2>&1; then
    success "Backend is running"
else
    error "Backend is not running at $BASE_URL"
    echo ""
    echo "Please start the backend first:"
    echo "  cd ppdb-sekolah/backend"
    echo "  cargo run"
    exit 1
fi

echo ""
echo "Press Enter to start complete flow test or Ctrl+C to cancel..."
read

echo ""
echo "=============================================="
echo "Running Complete End-to-End Flow Test"
echo "=============================================="
echo ""

# Run the complete flow test
if bash ./complete-flow-test.sh; then
    echo ""
    echo "=============================================="
    echo "✓ ALL TESTS PASSED!"
    echo "=============================================="
    echo ""
    echo "Test coverage:"
    echo "  ✓ Authentication flow"
    echo "  ✓ Multi-tenant isolation"
    echo "  ✓ RBAC permissions"
    echo "  ✓ Period & path management"
    echo "  ✓ Student registration"
    echo "  ✓ Document verification"
    echo "  ✓ Score calculation"
    echo "  ✓ Selection & ranking"
    echo "  ✓ Result announcement"
    echo ""
    echo "Test data saved in: .test-env"
    echo "View API docs: http://localhost:8000/swagger-ui/"
    echo ""
    exit 0
else
    echo ""
    echo "=============================================="
    echo "✗ TESTS FAILED"
    echo "=============================================="
    echo ""
    echo "Check the error messages above for details."
    echo "Common issues:"
    echo "  - Backend not fully started"
    echo "  - Database not migrated"
    echo "  - Seed data not loaded"
    echo ""
    echo "To fix:"
    echo "  1. cd ppdb-sekolah/backend"
    echo "  2. sqlx database reset -y"
    echo "  3. cargo run"
    echo "  4. Run tests again"
    echo ""
    exit 1
fi
