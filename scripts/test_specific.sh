#!/bin/bash

# PPDB Backend - Run Specific Test Function
# Usage: ./scripts/test_specific.sh <test_function_name>

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

if [ -z "$1" ]; then
    echo -e "${RED}Error: Test function name is required${NC}"
    echo ""
    echo -e "${YELLOW}Usage:${NC}"
    echo "  ./scripts/test_specific.sh <test_function_name>"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo "  ./scripts/test_specific.sh test_auth_login_success"
    echo "  ./scripts/test_specific.sh test_school_create_success"
    echo "  ./scripts/test_specific.sh test_registration_create_success"
    exit 1
fi

TEST_NAME=$1

echo -e "${BLUE}🧪 Running Specific Test: ${TEST_NAME}${NC}"
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
echo -e "${BLUE}🏃 Running test: ${TEST_NAME}...${NC}"
echo ""

# Run the specific test
cargo test ${TEST_NAME} -- --test-threads=1 --nocapture --exact || {
    echo -e "${RED}❌ Test failed: ${TEST_NAME}${NC}"
    exit 1
}

echo ""
echo -e "${GREEN}✅ Test '${TEST_NAME}' passed successfully!${NC}"
