# PPDB API Testing Guide

Dokumentasi testing untuk PPDB Sekolah API berdasarkan flow requirements di `sdlc/requirements.md`.

## Prerequisites

1. Backend server running di `http://localhost:8000`
2. Database sudah di-setup dan migrate
3. Swagger UI tersedia di `http://localhost:8000/swagger-ui/`

## Default Credentials (from seed data)

**SuperAdmin:**
- Email: `superadmin@ppdb.com`
- Password: `admin123`

**School Admin (SMA Negeri 1 Jakarta):**
- Email: `admin@sman1jkt.sch.id`
- Password: `admin123`

**Parent (Test User):**
- Email: `parent1@example.com`
- Password: `admin123`

## Testing Tools

- **Swagger UI**: Testing manual via browser
- **Postman/Thunder Client**: Import collection dari folder `postman/`
- **curl**: Command line testing (lihat `curl-examples/`)
- **Automated Tests**: Rust integration tests di `../ppdb-sekolah/backend/tests/`

## Test Flow Scenarios

Testing mengikuti flow bisnis PPDB:

### 1. Setup Multi-Tenant & RBAC
- Create school (SuperAdmin)
- Create admin sekolah
- Login sebagai admin sekolah
- Verify data isolation

### 2. Authentication Flow
- Register user (orang tua)
- Verify email
- Login
- Refresh token
- Logout

### 3. Period & Path Setup
- Create periode PPDB
- Create jalur pendaftaran (Zonasi, Prestasi, Afirmasi, Perpindahan Tugas)
- Activate periode

### 4. Registration Flow
- Create registration (draft)
- Upload documents
- Update registration data
- Submit registration

### 5. Verification Flow
- Admin view pending verifications
- Verify documents
- Approve/reject registration

### 6. Selection Flow
- Calculate scores
- Update rankings
- Run selection
- Announce results

### 7. Payment & Enrollment Flow
- Check payment info
- Process payment
- Verify payment
- Enrollment confirmation

## Test Scenarios by Requirement

Lihat file-file berikut untuk detail test cases:

- `01-multi-tenant-rbac.md` - Req 1: Multi-tenant & RBAC
- `02-authentication.md` - Req 2: User Management
- `03-complete-flow-test.md` - Complete end-to-end flow
- `SWAGGER_TESTING_GUIDE.md` - Detailed Swagger UI guide

## Quick Start

### 1. Test via Swagger UI

```bash
# Start backend
cd ppdb-sekolah/backend
cargo run

# Open browser
open http://localhost:8000/swagger-ui/
```

Login dengan credentials di atas, lalu ikuti panduan di `SWAGGER_TESTING_GUIDE.md`

### 2. Test via curl

```bash
# Run test scenarios
cd testing/curl-examples
./run-all-tests.sh
```

### 3. Test via Postman

```bash
# Import collection
# File -> Import -> testing/postman/PPDB-API-Collection.json
```

## Environment Variables

Buat file `.env` untuk testing:

```env
API_BASE_URL=http://localhost:8000
SUPER_ADMIN_EMAIL=superadmin@ppdb.com
SUPER_ADMIN_PASSWORD=admin123
TEST_SCHOOL_ID=1
TEST_ADMIN_EMAIL=admin@sman1jkt.sch.id
TEST_PARENT_EMAIL=parent1@example.com
```

## Test Data

Sample test data tersedia di folder `test-data/`:
- `sample-schools.json` - Sample schools
- `sample-registrations.json` - Sample registrations with various paths
- `documents/` - Sample document files (if needed)

## Seeded Data (Available After Migration)

Database sudah memiliki data awal:

**Schools:**
- SMA Negeri 1 Jakarta (ID: 1)
- SMP Negeri 5 Bandung (ID: 2)
- SD Negeri 10 Surabaya (ID: 3)

**Users:**
- SuperAdmin: superadmin@ppdb.com
- School Admins: admin@sman1jkt.sch.id, admin@smpn5bdg.sch.id, admin@sdn10sby.sch.id
- Parents: parent1@example.com, parent2@example.com

**Periods:**
- Active periods untuk semua schools (2025/2026)

**Registration Paths:**
- Zonasi, Prestasi, Afirmasi, Perpindahan Tugas untuk setiap period

## Automated Testing

Run integration tests:

```bash
cd ppdb-sekolah/backend
cargo test --test '*_integration_tests'
```

## Test Coverage

Target coverage per module:
- ✅ Authentication: 100%
- ✅ Schools: 100%
- ✅ Users: 100%
- ✅ Periods: 100%
- ✅ Registrations: 100%
- ✅ Verifications: 100%
- ✅ Selection: 100%
- ✅ Announcements: 100%

## Troubleshooting

### Common Issues

1. **401 Unauthorized**: Token expired, login ulang
2. **403 Forbidden**: Insufficient permissions, check role
3. **404 Not Found**: Resource tidak ada atau salah school_id
4. **422 Validation Error**: Check request body format

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug cargo run
```

## Contributing

Saat menambah endpoint baru:
1. Update Swagger documentation di `src/api/docs.rs`
2. Tambah test scenario di folder ini
3. Update Postman collection
4. Tambah integration test di `tests/`

## Notes

- Semua password di seed data: `admin123`
- Email verification di-skip untuk test users (sudah verified)
- Test dengan data realistis dari `test-data/`
- Verify data isolation antar schools
- Check audit logs untuk semua operations
