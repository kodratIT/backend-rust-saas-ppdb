#!/bin/bash

# PPDB Backend - Run All Integration Tests
# This script runs all integration tests for the PPDB backend

set -e

echo "🧪 PPDB Backend - Running All Integration Tests"
echo "================================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if .env file exists
if [ ! -f .env ]; then
    echo -e "${RED}Error: .env file not found!${NC}"
    echo "Please create .env file from .env.example"
    exit 1
fi

# Load environment variables
source .env

echo -e "${BLUE}📋 Test Configuration${NC}"
echo "Database: ${DATABASE_URL}"
echo "Port: ${PORT}"
echo ""

# Ensure database exists and migrations are run
echo -e "${BLUE}🔧 Preparing database...${NC}"
sqlx database create 2>/dev/null || true
sqlx migrate run

echo ""
echo -e "${BLUE}🏃 Running Integration Tests...${NC}"
echo ""

# Array of test modules
test_modules=(
    "auth_integration_tests"
    "school_integration_tests"
    "user_integration_tests"
    "period_integration_tests"
    "registration_integration_tests"
    "verification_integration_tests"
    "selection_integration_tests"
    "announcement_integration_tests"
)

# Run each test module
for module in "${test_modules[@]}"; do
    echo -e "${BLUE}Testing: ${module}${NC}"
    cargo test --test ${module} -- --test-threads=1 --nocapture || {
        echo -e "${RED}❌ Test failed: ${module}${NC}"
        exit 1
    }
    echo ""
done

echo ""
echo -e "${GREEN}✅ All Integration Tests Passed!${NC}"
echo ""

# Display test summary
echo -e "${BLUE}📊 Test Summary${NC}"
echo "================================"
echo "✅ Authentication Tests"
echo "✅ School Management Tests"
echo "✅ User Management Tests"
echo "✅ Period Management Tests"
echo "✅ Registration Tests"
echo "✅ Verification Tests"
echo "✅ Selection & Scoring Tests"
echo "✅ Announcement Tests"
echo ""
echo -e "${GREEN}🎉 All systems operational!${NC}"
