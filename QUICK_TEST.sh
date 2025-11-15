#!/bin/bash

# Quick Test Script untuk PPDB Backend
# Usage: ./QUICK_TEST.sh [test_name]

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== PPDB Backend Test Runner ===${NC}\n"

# Function to run test
run_test() {
    local test_name=$1
    local test_file=$2
    echo -e "${YELLOW}Running: ${test_name}${NC}"
    cargo test ${test_name} --test ${test_file} -- --test-threads=1 --nocapture
}

# Check argument
if [ $# -eq 0 ]; then
    echo "Available test commands:"
    echo ""
    echo -e "${GREEN}All Tests:${NC}"
    echo "  ./QUICK_TEST.sh all-school    # Semua test school management"
    echo "  ./QUICK_TEST.sh all-auth      # Semua test auth"
    echo "  ./QUICK_TEST.sh all           # Semua test (auth + school)"
    echo ""
    echo -e "${GREEN}School Tests:${NC}"
    echo "  ./QUICK_TEST.sh school-create"
    echo "  ./QUICK_TEST.sh school-list"
    echo "  ./QUICK_TEST.sh school-get"
    echo "  ./QUICK_TEST.sh school-update"
    echo "  ./QUICK_TEST.sh school-activate"
    echo "  ./QUICK_TEST.sh school-deactivate"
    echo ""
    echo -e "${GREEN}Auth Tests:${NC}"
    echo "  ./QUICK_TEST.sh auth-register"
    echo "  ./QUICK_TEST.sh auth-login"
    echo "  ./QUICK_TEST.sh auth-token"
    echo ""
    echo -e "${GREEN}User Tests:${NC}"
    echo "  ./QUICK_TEST.sh user-list"
    echo "  ./QUICK_TEST.sh user-get"
    echo "  ./QUICK_TEST.sh user-update"
    echo "  ./QUICK_TEST.sh user-password"
    echo "  ./QUICK_TEST.sh user-create"
    echo "  ./QUICK_TEST.sh user-delete"
    echo "  ./QUICK_TEST.sh user-rbac"
    echo ""
    exit 0
fi

case "$1" in
    # All tests
    all)
        echo -e "${GREEN}Running ALL tests...${NC}"
        cargo test --tests -- --test-threads=1 --nocapture
        ;;
    all-school)
        echo -e "${GREEN}Running ALL school tests...${NC}"
        cargo test --test school_integration_tests -- --test-threads=1 --nocapture
        ;;
    all-auth)
        echo -e "${GREEN}Running ALL auth tests...${NC}"
        cargo test --test auth_integration_tests -- --test-threads=1 --nocapture
        ;;
    all-user)
        echo -e "${GREEN}Running ALL user tests...${NC}"
        cargo test --test user_integration_tests -- --test-threads=1 --nocapture
        ;;
    
    # School tests
    school-create)
        run_test "test_school_create_success" "school_integration_tests"
        ;;
    school-list)
        run_test "test_school_list" "school_integration_tests"
        ;;
    school-get)
        run_test "test_school_get_by_id" "school_integration_tests"
        ;;
    school-update)
        run_test "test_school_update" "school_integration_tests"
        ;;
    school-activate)
        run_test "test_school_activate" "school_integration_tests"
        ;;
    school-deactivate)
        run_test "test_school_deactivate" "school_integration_tests"
        ;;
    
    # Auth tests
    auth-register)
        run_test "test_auth_register_success" "auth_integration_tests"
        ;;
    auth-login)
        run_test "test_auth_login_success" "auth_integration_tests"
        ;;
    auth-token)
        run_test "test_auth_refresh_token" "auth_integration_tests"
        ;;
    
    # User tests
    user-list)
        run_test "test_user_list_as_super_admin" "user_integration_tests"
        ;;
    user-get)
        run_test "test_user_get_by_id" "user_integration_tests"
        ;;
    user-update)
        run_test "test_user_update_profile" "user_integration_tests"
        ;;
    user-password)
        run_test "test_user_change_password_success" "user_integration_tests"
        ;;
    user-create)
        run_test "test_user_create_as_super_admin" "user_integration_tests"
        ;;
    user-delete)
        run_test "test_user_delete_as_super_admin" "user_integration_tests"
        ;;
    user-rbac)
        run_test "test_user_delete_as_parent_forbidden" "user_integration_tests"
        ;;
    
    *)
        echo -e "${YELLOW}Unknown test: $1${NC}"
        echo "Run './QUICK_TEST.sh' without arguments to see available tests"
        exit 1
        ;;
esac

echo -e "\n${GREEN}✓ Test completed!${NC}"
