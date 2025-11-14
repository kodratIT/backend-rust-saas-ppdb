#!/bin/bash

# PPDB Backend Test Runner
# This script runs all tests with proper setup

set -e

echo "🧪 PPDB Backend Test Runner"
echo "============================"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if .env.test exists
if [ ! -f .env.test ]; then
    echo -e "${YELLOW}⚠️  .env.test not found. Creating from .env.example...${NC}"
    cp .env.example .env.test
    echo "DATABASE_URL=postgresql://localhost/ppdb_test" >> .env.test
    echo -e "${GREEN}✓ Created .env.test${NC}"
fi

# Load test environment
export $(cat .env.test | xargs)

echo "📋 Test Configuration:"
echo "   Database: $DATABASE_URL"
echo ""

# Check if PostgreSQL is running
echo "🔍 Checking PostgreSQL connection..."
if ! psql "$DATABASE_URL" -c '\q' 2>/dev/null; then
    echo -e "${RED}❌ Cannot connect to PostgreSQL${NC}"
    echo "   Please ensure PostgreSQL is running and DATABASE_URL is correct"
    exit 1
fi
echo -e "${GREEN}✓ PostgreSQL connection OK${NC}"
echo ""

# Create test database if not exists
echo "🗄️  Setting up test database..."
psql -U postgres -tc "SELECT 1 FROM pg_database WHERE datname = 'ppdb_test'" | grep -q 1 || \
    psql -U postgres -c "CREATE DATABASE ppdb_test"
echo -e "${GREEN}✓ Test database ready${NC}"
echo ""

# Run migrations
echo "🔄 Running migrations..."
sqlx migrate run
echo -e "${GREEN}✓ Migrations completed${NC}"
echo ""

# Run tests
echo "🧪 Running tests..."
echo ""

if [ "$1" == "unit" ]; then
    echo "Running unit tests only..."
    cargo test --lib
elif [ "$1" == "integration" ]; then
    echo "Running integration tests only..."
    cargo test --test '*'
elif [ "$1" == "coverage" ]; then
    echo "Running tests with coverage..."
    cargo tarpaulin --out Html --output-dir coverage
else
    echo "Running all tests..."
    cargo test -- --test-threads=1
fi

TEST_EXIT_CODE=$?

echo ""
if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
else
    echo -e "${RED}❌ Some tests failed${NC}"
fi

echo ""
echo "📊 Test Summary:"
echo "   Exit code: $TEST_EXIT_CODE"
echo ""

exit $TEST_EXIT_CODE
