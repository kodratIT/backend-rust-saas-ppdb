# API Testing Guide - PPDB Backend

## Prerequisites
- Rust 1.70+ installed
- PostgreSQL database running
- Environment variables configured in `.env`

## Setup

1. Install dependencies:
```bash
cargo build
```

2. Run migrations:
```bash
sqlx migrate run
```

3. Start server:
```bash
cargo run
```

Server will run at `http://localhost:8080`

## API Endpoints Summary

### Base URL
```
http://localhost:8080/api/v1
```

---

## Phase 3: Authentication & Authorization

### 1. Register User (Parent)
```bash
POST /auth/register
Content-Type: application/json

{
  "email": "parent@example.com",
  "password": "password123",
  "full_name": "John Doe",
  "phone": "081234567890",
  "nik": "1234567890123456"
}

# Expected: 201 Created
# Response: UserResponse with id, email, role=parent
```

### 2. Login
```bash
POST /auth/login
Content-Type: application/json

{
  "email": "parent@example.com",
  "password": "password123"
}

# Expected: 200 OK
# Response: 
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {...}
}
```

### 3. Refresh Token
```bash
POST /auth/refresh
Content-Type: application/json

{
  "refresh_token": "eyJ..."
}

# Expected: 200 OK
# Response: New access_token
```

### 4. Get Current User
```bash
GET /auth/me
Authorization: Bearer <access_token>

# Expected: 200 OK
# Response: UserResponse
```

### 5. Logout
```bash
POST /auth/logout
Authorization: Bearer <access_token>

# Expected: 200 OK
```

---

## Phase 4: School & User Management

### 6. Create School (Super Admin Only)
```bash
POST /schools
Authorization: Bearer <super_admin_token>
Content-Type: application/json

{
  "name": "SMA Negeri 1 Jakarta",
  "npsn": "12345678",
  "code": "SMAN1JKT",
  "address": "Jl. Pendidikan No. 1",
  "phone": "021-1234567",
  "email": "info@sman1jkt.sch.id"
}

# Expected: 201 Created
# Response: SchoolResponse
```

### 7. List Schools (Super Admin Only)
```bash
GET /schools?page=1&page_size=10&status=active
Authorization: Bearer <super_admin_token>

# Expected: 200 OK
# Response: Paginated list of schools
```

### 8. Create User (Admin Only)
```bash
POST /users
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "email": "admin@school.com",
  "password": "password123",
  "full_name": "School Admin",
  "role": "school_admin",
  "school_id": 1
}

# Expected: 201 Created
# Response: UserResponse
```

### 9. List Users
```bash
GET /users?page=1&page_size=10&role=parent
Authorization: Bearer <admin_token>

# Expected: 200 OK
# Response: Paginated list of users (filtered by school for school_admin)
```

---

## Phase 5: Period & Registration Path Management

### 10. Create Period with Paths (School Admin Only)
```bash
POST /periods
Authorization: Bearer <school_admin_token>
Content-Type: application/json

{
  "academic_year": "2024/2025",
  "level": "SMA",
  "start_date": "2024-01-01T00:00:00Z",
  "end_date": "2024-03-31T23:59:59Z",
  "reenrollment_deadline": "2024-04-15T23:59:59Z",
  "paths": [
    {
      "path_type": "zonasi",
      "name": "Jalur Zonasi",
      "quota": 100,
      "description": "Jalur zonasi berdasarkan jarak tempat tinggal",
      "scoring_config": {
        "distance_weight": 2.0
      }
    },
    {
      "path_type": "prestasi",
      "name": "Jalur Prestasi",
      "quota": 50,
      "description": "Jalur prestasi akademik",
      "scoring_config": {
        "rapor_weight": 0.7,
        "achievement_weight": 0.3
      }
    }
  ]
}

# Expected: 201 Created
# Response: PeriodWithPathsResponse
```

### 11. List Periods
```bash
GET /periods?page=1&page_size=10&status=active
Authorization: Bearer <school_admin_token>

# Expected: 200 OK
# Response: Paginated list of periods
```

### 12. Activate Period
```bash
POST /periods/:id/activate
Authorization: Bearer <school_admin_token>

# Expected: 200 OK
# Response: Updated period with status=active
# Note: Will deactivate other active periods for same school/year/level
```

---

## Phase 6: Student Registration & Document Upload

### 13. Create Registration (Parent)
```bash
POST /registrations
Authorization: Bearer <parent_token>
Content-Type: application/json

{
  "period_id": 1,
  "path_id": 1,
  "student_nisn": "1234567890",
  "student_name": "Ahmad Rizki",
  "student_gender": "L",
  "student_birth_place": "Jakarta",
  "student_birth_date": "2010-05-15T00:00:00Z",
  "student_religion": "Islam",
  "student_address": "Jl. Merdeka No. 10",
  "student_phone": "081234567890",
  "parent_name": "Budi Santoso",
  "parent_nik": "3201234567890123",
  "parent_phone": "081234567890",
  "path_data": {
    "distance_km": 2.5
  }
}

# Expected: 201 Created
# Response: RegistrationResponse with status=draft
```

### 14. Update Registration (Draft Only)
```bash
PUT /registrations/:id
Authorization: Bearer <parent_token>
Content-Type: application/json

{
  "student_name": "Ahmad Rizki Updated",
  "student_address": "Jl. Merdeka No. 11"
}

# Expected: 200 OK
# Response: Updated RegistrationResponse
```

### 15. Upload Document
```bash
POST /registrations/:id/documents
Authorization: Bearer <parent_token>
Content-Type: application/json

{
  "document_type": "kartu_keluarga",
  "file_url": "https://storage.example.com/kk.pdf",
  "file_name": "kartu_keluarga.pdf",
  "file_size": 1048576,
  "mime_type": "application/pdf"
}

# Expected: 201 Created
# Response: DocumentResponse
# Note: File size max 2MB, types: JPEG, PNG, PDF
```

### 16. Submit Registration
```bash
POST /registrations/:id/submit
Authorization: Bearer <parent_token>

# Expected: 200 OK
# Response: RegistrationResponse with status=submitted, registration_number generated
# Validation: Must have documents uploaded
```

### 17. List My Registrations (Parent)
```bash
GET /registrations?page=1&page_size=10
Authorization: Bearer <parent_token>

# Expected: 200 OK
# Response: List of user's registrations only
```

---

## Phase 7: Document Verification & Admin Review

### 18. Get Pending Verifications (Admin Only)
```bash
GET /verifications/pending?page=1&page_size=10&period_id=1
Authorization: Bearer <school_admin_token>

# Expected: 200 OK
# Response: List of submitted registrations
```

### 19. Verify Registration (Admin Only)
```bash
POST /verifications/:id/verify
Authorization: Bearer <school_admin_token>

# Expected: 200 OK
# Response: RegistrationResponse with status=verified
```

### 20. Reject Registration (Admin Only)
```bash
POST /verifications/:id/reject
Authorization: Bearer <school_admin_token>
Content-Type: application/json

{
  "reason": "Dokumen tidak lengkap atau tidak sesuai persyaratan"
}

# Expected: 200 OK
# Response: RegistrationResponse with status=rejected
# Note: Reason min 10 characters
```

### 21. Get Verification Stats (Admin Only)
```bash
GET /verifications/stats?period_id=1
Authorization: Bearer <school_admin_token>

# Expected: 200 OK
# Response: 
{
  "total": 150,
  "submitted": 30,
  "verified": 100,
  "rejected": 20,
  "pending": 30
}
```

---

## Phase 8: Selection Score Calculation & Ranking

### 22. Calculate Scores (Admin Only)
```bash
POST /selection/periods/:period_id/calculate-scores
Authorization: Bearer <school_admin_token>

# Expected: 200 OK
# Response: 
{
  "message": "Successfully calculated scores for 100 registrations",
  "total_calculated": 100
}
# Note: Calculates scores for all verified registrations
```

### 23. Update Rankings (Admin Only)
```bash
POST /selection/periods/:period_id/update-rankings
Authorization: Bearer <school_admin_token>

# Expected: 200 OK
# Response: 
{
  "message": "Successfully updated rankings for 100 registrations",
  "total_ranked": 100
}
# Note: Rankings per path, ordered by score DESC
```

### 24. Get Rankings (Admin Only)
```bash
GET /selection/periods/:period_id/rankings?path_id=1&page=1&page_size=50
Authorization: Bearer <school_admin_token>

# Expected: 200 OK
# Response: List of registrations with scores and rankings
```

### 25. Get Ranking Statistics (Admin Only)
```bash
GET /selection/periods/:period_id/stats
Authorization: Bearer <school_admin_token>

# Expected: 200 OK
# Response: Array of path statistics with highest, lowest, average scores
```

---

## Phase 9: Final Selection & Announcement

### 26. Run Selection (Admin Only)
```bash
POST /announcements/periods/:period_id/run-selection
Authorization: Bearer <school_admin_token>

# Expected: 200 OK
# Response: 
{
  "message": "Selection completed successfully. 100 accepted, 50 rejected",
  "result": {
    "total_accepted": 100,
    "total_rejected": 50
  }
}
# Note: Top N by ranking = accepted, others = rejected (N = quota)
```

### 27. Announce Results (Admin Only)
```bash
POST /announcements/periods/:period_id/announce
Authorization: Bearer <school_admin_token>

# Expected: 200 OK
# Response: 
{
  "message": "Results announced successfully. 150 notifications sent",
  "result": {
    "total_notified": 150,
    "accepted_notified": 100,
    "rejected_notified": 50
  }
}
# Note: Updates announcement_date, sends notifications
```

### 28. Get Selection Summary (Admin Only)
```bash
GET /announcements/periods/:period_id/summary
Authorization: Bearer <school_admin_token>

# Expected: 200 OK
# Response: Overall and per-path selection statistics
```

### 29. Check Result (Public - No Auth)
```bash
GET /announcements/check-result?registration_number=REG-1-1-00001&student_nisn=1234567890

# Expected: 200 OK
# Response: 
{
  "registration_number": "REG-1-1-00001",
  "student_name": "Ahmad Rizki",
  "student_nisn": "1234567890",
  "path_name": "Jalur Zonasi",
  "selection_score": 95.0,
  "ranking": 5,
  "status": "accepted",
  "rejection_reason": null,
  "announcement_date": "2024-04-01T00:00:00Z",
  "reenrollment_deadline": "2024-04-15T23:59:59Z"
}
# Note: Only works after announcement_date
```

---

## Testing Workflow

### Complete PPDB Flow Test

1. **Setup (Super Admin)**
   - Create school
   - Create school admin user

2. **Period Setup (School Admin)**
   - Create period with paths
   - Activate period

3. **Registration (Parent)**
   - Register parent account
   - Login
   - Create registration (draft)
   - Upload documents
   - Submit registration

4. **Verification (School Admin)**
   - View pending verifications
   - Verify or reject registrations

5. **Selection (School Admin)**
   - Calculate scores for all verified
   - Update rankings
   - View rankings and stats
   - Run selection
   - Announce results

6. **Result Check (Public)**
   - Check result by registration_number + NISN

---

## Expected Validations

### Authentication
- ✅ JWT token required for protected endpoints
- ✅ Token expiration (24 hours)
- ✅ Refresh token mechanism (7 days)
- ✅ Role-based access control

### Registration
- ✅ NISN must be 10 digits
- ✅ NIK must be 16 digits
- ✅ Period must be active
- ✅ Only draft can be updated
- ✅ Documents required before submit

### Verification
- ✅ Only submitted can be verified/rejected
- ✅ Rejection reason required (min 10 chars)
- ✅ Admin only access

### Selection
- ✅ Only verified registrations scored
- ✅ Rankings per path
- ✅ Quota enforcement
- ✅ Only active periods can run selection

### Multi-tenant
- ✅ School admin sees only their school data
- ✅ Parents see only their registrations
- ✅ Super admin sees all schools

---

## Error Responses

### 400 Bad Request
```json
{
  "error": "Validation error: NISN must be 10 digits"
}
```

### 401 Unauthorized
```json
{
  "error": "Not authenticated"
}
```

### 403 Forbidden
```json
{
  "error": "You don't have permission to access this resource"
}
```

### 404 Not Found
```json
{
  "error": "Registration not found"
}
```

### 409 Conflict
```json
{
  "error": "Email already registered"
}
```

---

## Performance Notes

- Pagination default: 10 items per page
- Max page size: 100 items
- Database connection pool: 20 connections
- File upload max size: 2MB
- Supported file types: JPEG, PNG, PDF

---

## Next Steps for Production

1. **Install Rust & Cargo**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Setup Database**
   - Create PostgreSQL database
   - Configure DATABASE_URL in .env
   - Run migrations

3. **Run Tests**
   ```bash
   cargo test
   ```

4. **Start Server**
   ```bash
   cargo run --release
   ```

5. **Test with Postman/Insomnia**
   - Import API collection
   - Test each endpoint
   - Verify business logic

6. **Deploy to Fly.io**
   ```bash
   fly deploy
   ```
