#!/bin/bash

# PPDB API Testing - Setup Flow
# Test complete setup dari SuperAdmin create school hingga period activation

set -e

BASE_URL="http://localhost:8000"
SUPERADMIN_EMAIL="superadmin@ppdb.com"
SUPERADMIN_PASSWORD="admin123"

echo "========================================="
echo "PPDB API Testing - Setup Flow"
echo "========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print success
success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Function to print error
error() {
    echo -e "${RED}✗ $1${NC}"
}

# Function to print info
info() {
    echo -e "${YELLOW}→ $1${NC}"
}

# Step 1: SuperAdmin Login
info "Step 1: SuperAdmin Login"
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$SUPERADMIN_EMAIL\",
    \"password\": \"$SUPERADMIN_PASSWORD\"
  }")

SUPERADMIN_TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.access_token')

if [ "$SUPERADMIN_TOKEN" != "null" ] && [ -n "$SUPERADMIN_TOKEN" ]; then
    success "SuperAdmin logged in successfully"
    echo "Token: ${SUPERADMIN_TOKEN:0:20}..."
else
    error "SuperAdmin login failed"
    echo $LOGIN_RESPONSE | jq '.'
    exit 1
fi

echo ""

# Step 2: Create School
info "Step 2: Create School"
SCHOOL_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/schools" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $SUPERADMIN_TOKEN" \
  -d "{
    \"name\": \"SMA Test $(date +%s)\",
    \"npsn\": \"$(date +%s | tail -c 9)\",
    \"code\": \"SMATEST$(date +%s | tail -c 5)\",
    \"address\": \"Jl. Test No. 1\",
    \"phone\": \"021-9999999\",
    \"email\": \"test$(date +%s)@test.sch.id\"
  }")

SCHOOL_ID=$(echo $SCHOOL_RESPONSE | jq -r '.id')

if [ "$SCHOOL_ID" != "null" ] && [ -n "$SCHOOL_ID" ]; then
    success "School created successfully"
    echo "School ID: $SCHOOL_ID"
    echo "School Name: $(echo $SCHOOL_RESPONSE | jq -r '.name')"
    echo "NPSN: $(echo $SCHOOL_RESPONSE | jq -r '.npsn')"
else
    error "School creation failed"
    echo $SCHOOL_RESPONSE | jq '.'
    exit 1
fi

echo ""

# Step 3: Create School Admin
info "Step 3: Create School Admin"
ADMIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/users" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $SUPERADMIN_TOKEN" \
  -d "{
    \"email\": \"admin@sman1jaktest.sch.id\",
    \"password\": \"admin123\",
    \"full_name\": \"Admin SMA 1 Jakarta Test\",
    \"phone\": \"081234567890\",
    \"role\": \"school_admin\",
    \"school_id\": $SCHOOL_ID
  }")

ADMIN_ID=$(echo $ADMIN_RESPONSE | jq -r '.id')

if [ "$ADMIN_ID" != "null" ] && [ -n "$ADMIN_ID" ]; then
    success "School Admin created successfully"
    echo "Admin ID: $ADMIN_ID"
    echo "Admin Email: $(echo $ADMIN_RESPONSE | jq -r '.email')"
else
    error "Admin creation failed"
    echo $ADMIN_RESPONSE | jq '.'
    exit 1
fi

echo ""

# Step 4: Admin Login
info "Step 4: School Admin Login"
ADMIN_LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@sman1jaktest.sch.id",
    "password": "admin123"
  }')

ADMIN_TOKEN=$(echo $ADMIN_LOGIN_RESPONSE | jq -r '.access_token')

if [ "$ADMIN_TOKEN" != "null" ] && [ -n "$ADMIN_TOKEN" ]; then
    success "School Admin logged in successfully"
    echo "Token: ${ADMIN_TOKEN:0:20}..."
else
    error "Admin login failed"
    echo $ADMIN_LOGIN_RESPONSE | jq '.'
    exit 1
fi

echo ""

# Step 5: Create PPDB Period
info "Step 5: Create PPDB Period"
PERIOD_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/periods" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{
    "name": "PPDB 2024/2025",
    "academic_year": "2024/2025",
    "level": "SMA",
    "start_date": "2024-06-01T00:00:00Z",
    "end_date": "2024-07-31T23:59:59Z",
    "registration_start": "2024-06-01T00:00:00Z",
    "registration_end": "2024-06-30T23:59:59Z",
    "announcement_date": "2024-07-15T00:00:00Z"
  }')

PERIOD_ID=$(echo $PERIOD_RESPONSE | jq -r '.id')

if [ "$PERIOD_ID" != "null" ] && [ -n "$PERIOD_ID" ]; then
    success "PPDB Period created successfully"
    echo "Period ID: $PERIOD_ID"
    echo "Period Name: $(echo $PERIOD_RESPONSE | jq -r '.name')"
    echo "Academic Year: $(echo $PERIOD_RESPONSE | jq -r '.academic_year')"
else
    error "Period creation failed"
    echo $PERIOD_RESPONSE | jq '.'
    exit 1
fi

echo ""

# Step 6: Create Registration Paths
info "Step 6: Create Registration Paths"

# Jalur Zonasi
ZONASI_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/periods/$PERIOD_ID/paths" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{
    "name": "Jalur Zonasi",
    "path_type": "Zonasi",
    "quota": 180,
    "description": "Jalur zonasi berdasarkan jarak tempat tinggal"
  }')

ZONASI_PATH_ID=$(echo $ZONASI_RESPONSE | jq -r '.id')

if [ "$ZONASI_PATH_ID" != "null" ] && [ -n "$ZONASI_PATH_ID" ]; then
    success "Jalur Zonasi created (ID: $ZONASI_PATH_ID, Quota: 180)"
else
    error "Jalur Zonasi creation failed"
fi

# Jalur Prestasi
PRESTASI_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/periods/$PERIOD_ID/paths" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{
    "name": "Jalur Prestasi",
    "path_type": "Prestasi",
    "quota": 90,
    "description": "Jalur prestasi akademik dan non-akademik"
  }')

PRESTASI_PATH_ID=$(echo $PRESTASI_RESPONSE | jq -r '.id')

if [ "$PRESTASI_PATH_ID" != "null" ] && [ -n "$PRESTASI_PATH_ID" ]; then
    success "Jalur Prestasi created (ID: $PRESTASI_PATH_ID, Quota: 90)"
else
    error "Jalur Prestasi creation failed"
fi

# Jalur Afirmasi
AFIRMASI_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/periods/$PERIOD_ID/paths" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{
    "name": "Jalur Afirmasi",
    "path_type": "Afirmasi",
    "quota": 20,
    "description": "Jalur afirmasi untuk siswa kurang mampu"
  }')

AFIRMASI_PATH_ID=$(echo $AFIRMASI_RESPONSE | jq -r '.id')

if [ "$AFIRMASI_PATH_ID" != "null" ] && [ -n "$AFIRMASI_PATH_ID" ]; then
    success "Jalur Afirmasi created (ID: $AFIRMASI_PATH_ID, Quota: 20)"
else
    error "Jalur Afirmasi creation failed"
fi

# Jalur Perpindahan Tugas
PERPINDAHAN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/periods/$PERIOD_ID/paths" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{
    "name": "Jalur Perpindahan Tugas Orang Tua",
    "path_type": "PerpindahanTugas",
    "quota": 10,
    "description": "Jalur perpindahan tugas orang tua/wali"
  }')

PERPINDAHAN_PATH_ID=$(echo $PERPINDAHAN_RESPONSE | jq -r '.id')

if [ "$PERPINDAHAN_PATH_ID" != "null" ] && [ -n "$PERPINDAHAN_PATH_ID" ]; then
    success "Jalur Perpindahan Tugas created (ID: $PERPINDAHAN_PATH_ID, Quota: 10)"
else
    error "Jalur Perpindahan Tugas creation failed"
fi

echo ""

# Step 7: Activate Period
info "Step 7: Activate PPDB Period"
ACTIVATE_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/periods/$PERIOD_ID/activate" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

PERIOD_STATUS=$(echo $ACTIVATE_RESPONSE | jq -r '.status')

if [ "$PERIOD_STATUS" == "Active" ]; then
    success "PPDB Period activated successfully"
else
    error "Period activation failed"
    echo $ACTIVATE_RESPONSE | jq '.'
fi

echo ""
echo "========================================="
echo "Setup Flow Completed Successfully!"
echo "========================================="
echo ""
echo "Summary:"
echo "  School ID: $SCHOOL_ID"
echo "  Admin Token: ${ADMIN_TOKEN:0:30}..."
echo "  Period ID: $PERIOD_ID"
echo "  Zonasi Path ID: $ZONASI_PATH_ID"
echo "  Prestasi Path ID: $PRESTASI_PATH_ID"
echo "  Afirmasi Path ID: $AFIRMASI_PATH_ID"
echo "  Perpindahan Path ID: $PERPINDAHAN_PATH_ID"
echo ""
echo "Save these IDs for next test scenarios!"
echo ""

# Save to file for next scripts
cat > .test-env << EOF
SCHOOL_ID=$SCHOOL_ID
ADMIN_TOKEN=$ADMIN_TOKEN
PERIOD_ID=$PERIOD_ID
ZONASI_PATH_ID=$ZONASI_PATH_ID
PRESTASI_PATH_ID=$PRESTASI_PATH_ID
AFIRMASI_PATH_ID=$AFIRMASI_PATH_ID
PERPINDAHAN_PATH_ID=$PERPINDAHAN_PATH_ID
EOF

success "Test environment saved to .test-env"
