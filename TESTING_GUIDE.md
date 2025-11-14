# Testing Guide - PPDB Backend

## 🎯 Overview

This guide provides comprehensive testing instructions for the PPDB Backend API. We have implemented both **automated tests** and **manual testing tools** to ensure code quality and functionality.

## 📋 Table of Contents

1. [Prerequisites](#prerequisites)
2. [Automated Testing](#automated-testing)
3. [Manual Testing](#manual-testing)
4. [Test Coverage](#test-coverage)
5. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required Software
- **Rust** 1.70+ ([Install](https://rustup.rs))
- **PostgreSQL** 14+
- **SQLx CLI**: `cargo install sqlx-cli --no-default-features --features postgres`

### Database Setup
```bash
# Create test database
createdb ppdb_test

# Or using psql
psql -U postgres -c "CREATE DATABASE ppdb_test"
```

### Environment Configuration
```bash
# Copy environment template
cp .env.example .env.test

# Edit .env.test
DATABASE_URL=postgresql://localhost/ppdb_test
JWT_SECRET=test-secret-key-for-testing
JWT_EXPIRATION_HOURS=24
PORT=8080
```

---

## 🤖 Automated Testing

### Quick Start

```bash
# Run all tests
./scripts/run_tests.sh

# Run unit tests only
./scripts/run_tests.sh unit

# Run integration tests only
./scripts/run_tests.sh integration

# Run with coverage report
./scripts/run_tests.sh coverage
```

### Manual Test Execution

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test auth_tests

# Run specific test
cargo test test_register_user_success

# Run with output
cargo test -- --nocapture

# Run tests sequentially (avoid database conflicts)
cargo test -- --test-threads=1
```

### Test Structure

```
tests/
├── common/
│   └── mod.rs              # Test helpers and utilities
├── auth_tests.rs           # Authentication tests
├── registration_tests.rs   # Registration flow tests
└── ...                     # More test files
```

### Available Test Suites

#### 1. Authentication Tests (`auth_tests.rs`)
- ✅ User registration
- ✅ Duplicate email validation
- ✅ Login with valid credentials
- ✅ Login with invalid credentials
- ✅ Get current user
- ✅ Refresh token
- ✅ Unauthorized access
- ✅ Invalid token handling

#### 2. Registration Tests (`registration_tests.rs`)
- ✅ Create registration
- ✅ NISN validation (10 digits)
- ✅ NIK validation (16 digits)
- ✅ Submit registration
- ✅ Multi-tenant isolation (parents see only their own)

### Writing New Tests

```rust
// Example test
#[tokio::test]
async fn test_my_feature() {
    let state = create_test_app_state().await;
    let mut router = create_test_router(state.clone());
    
    cleanup_test_data(&state.db).await;
    
    // Your test logic here
    let (status, body) = make_json_request(
        &mut router,
        "POST",
        "/api/endpoint",
        Some(json!({"key": "value"})),
        None
    ).await;
    
    assert_eq!(status, StatusCode::OK);
}
```

---

## 🔧 Manual Testing

### Option 1: Postman Collection

1. **Import Collection**
   - Open Postman
   - Import `postman/PPDB_API_Collection.json`
   - Collection includes 30+ pre-configured requests

2. **Setup Environment**
   - Create new environment in Postman
   - Add variable: `base_url` = `http://localhost:8080/api/v1`

3. **Run Requests**
   - Start with "01. Authentication" folder
   - Variables are auto-populated (tokens, IDs)
   - Follow the numbered folders in order

### Option 2: cURL Commands

#### Authentication
```bash
# Register
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123",
    "full_name": "Test User",
    "phone": "081234567890",
    "nik": "1234567890123456"
  }'

# Login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }'

# Get current user
curl -X GET http://localhost:8080/api/v1/auth/me \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Option 3: HTTPie (Recommended)

```bash
# Install HTTPie
brew install httpie  # macOS
# or
pip install httpie

# Register
http POST :8080/api/v1/auth/register \
  email=test@example.com \
  password=password123 \
  full_name="Test User" \
  phone=081234567890 \
  nik=1234567890123456

# Login
http POST :8080/api/v1/auth/login \
  email=test@example.com \
  password=password123

# Get current user
http GET :8080/api/v1/auth/me \
  Authorization:"Bearer YOUR_TOKEN"
```

---

## 📊 Test Coverage

### Current Coverage

| Module | Coverage | Status |
|--------|----------|--------|
| Authentication | 80% | ✅ Good |
| Registration | 75% | ✅ Good |
| Verification | 60% | ⚠️ Needs improvement |
| Selection | 50% | ⚠️ Needs improvement |
| Announcement | 40% | ⚠️ Needs improvement |

### Generate Coverage Report

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML report
cargo tarpaulin --out Html --output-dir coverage

# Open report
open coverage/index.html
```

---

## 🧪 Complete Testing Workflow

### 1. Setup Phase
```bash
# Start PostgreSQL
brew services start postgresql  # macOS
# or
sudo systemctl start postgresql  # Linux

# Create test database
createdb ppdb_test

# Run migrations
sqlx migrate run
```

### 2. Run Automated Tests
```bash
# Run all tests
./scripts/run_tests.sh

# Check results
echo $?  # Should be 0 if all passed
```

### 3. Start Development Server
```bash
# Terminal 1: Start server
cargo run

# Terminal 2: Run manual tests
# Use Postman or cURL
```

### 4. Test Complete Flow

**A. Setup (Super Admin)**
1. Create school
2. Create school admin user

**B. Period Setup (School Admin)**
1. Login as school admin
2. Create period with paths
3. Activate period

**C. Registration (Parent)**
1. Register parent account
2. Login
3. Create registration (draft)
4. Upload documents
5. Submit registration

**D. Verification (School Admin)**
1. View pending verifications
2. Verify or reject registrations

**E. Selection (School Admin)**
1. Calculate scores
2. Update rankings
3. View rankings
4. Run selection
5. Announce results

**F. Result Check (Public)**
1. Check result by registration_number + NISN

---

## 🐛 Troubleshooting

### Common Issues

#### 1. Database Connection Error
```
Error: Failed to connect to database
```

**Solution:**
```bash
# Check PostgreSQL is running
pg_isready

# Check DATABASE_URL
echo $DATABASE_URL

# Restart PostgreSQL
brew services restart postgresql
```

#### 2. Migration Error
```
Error: Migration failed
```

**Solution:**
```bash
# Reset database
dropdb ppdb_test
createdb ppdb_test

# Run migrations again
sqlx migrate run
```

#### 3. Test Failures
```
Error: Test failed with database conflict
```

**Solution:**
```bash
# Run tests sequentially
cargo test -- --test-threads=1

# Clean test data
psql ppdb_test -c "TRUNCATE TABLE registrations, documents, periods, users, schools CASCADE"
```

#### 4. Port Already in Use
```
Error: Address already in use (os error 48)
```

**Solution:**
```bash
# Find process using port 8080
lsof -i :8080

# Kill process
kill -9 <PID>
```

---

## 📝 Test Checklist

### Before Committing Code

- [ ] All automated tests pass
- [ ] New features have tests
- [ ] Test coverage > 70%
- [ ] Manual testing completed
- [ ] No console errors
- [ ] Database migrations work
- [ ] API documentation updated

### Before Deployment

- [ ] All tests pass in CI/CD
- [ ] Integration tests pass
- [ ] Performance tests pass
- [ ] Security tests pass
- [ ] Load tests pass (if applicable)
- [ ] Rollback plan tested

---

## 🚀 CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: ppdb_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run tests
        run: cargo test
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost/ppdb_test
```

---

## 📚 Additional Resources

- [API Testing Guide](./API_TESTING.md) - Complete API endpoint documentation
- [Implementation Summary](./IMPLEMENTATION_SUMMARY.md) - Feature completion status
- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Axum Testing Examples](https://github.com/tokio-rs/axum/tree/main/examples)

---

## 🎉 Summary

**Testing Infrastructure Complete!**

- ✅ Automated test suite with 15+ tests
- ✅ Test helpers and utilities
- ✅ Postman collection with 30+ requests
- ✅ Test runner script
- ✅ Coverage reporting
- ✅ CI/CD ready

**Next Steps:**
1. Run `./scripts/run_tests.sh` to verify setup
2. Import Postman collection for manual testing
3. Add more tests as you develop new features
4. Maintain test coverage > 70%

Happy Testing! 🧪
