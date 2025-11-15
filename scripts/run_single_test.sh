#!/bin/bash

# PPDB Backend - Run Single Test Module
# Usage: ./scripts/run_single_test.sh <test_module_name>

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

if [ -z "$1" ]; then
    echo -e "${RED}Error: Test module name is required${NC}"
    echo ""
    echo -e "${YELLOW}Usage:${NC}"
    echo "  ./scripts/run_single_test.sh <test_module_name>"
    echo ""
    echo -e "${YELLOW}Available test modules:${NC}"
    echo "  - auth_integration_tests"
    echo "  - school_integration_tests"
    echo "  - user_integration_tests"
    echo "  - period_integration_tests"
    echo "  - registration_integration_tests"
    echo "  - verification_integration_tests"
    echo "  - selection_integration_tests"
    echo "  - announcement_integration_tests"
    echo ""
    echo -e "${YELLOW}Example:${NC}"
    echo "  ./scripts/run_single_test.sh auth_integration_tests"
    exit 1
fi

TEST_MODULE=$1

echo -e "${BLUE}🧪 Running Test Module: ${TEST_MODULE}${NC}"
echo "================================================"
echo ""

# Check if .env file exists
if [ ! -f .env ]; then
    echo -e "${RED}Error: .env file not found!${NC}"
    echo "Please create .env file from .env.example"
    exit 1
fi

# Load environment variables
source .env

# Ensure database exists and migrations are run
echo -e "${BLUE}🔧 Preparing database...${NC}"
sqlx database create 2>/dev/null || true
sqlx migrate run

echo ""
echo -e "${BLUE}🏃 Running ${TEST_MODULE}...${NC}"
echo ""

# Run the specified test module
cargo test --test ${TEST_MODULE} -- --test-threads=1 --nocapture || {
    echo -e "${RED}❌ Test failed: ${TEST_MODULE}${NC}"
    exit 1
}

echo ""
echo -e "${GREEN}✅ Test module '${TEST_MODULE}' passed successfully!${NC}"
