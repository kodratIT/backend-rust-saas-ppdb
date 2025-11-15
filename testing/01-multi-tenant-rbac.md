# Test Scenario 1: Multi-Tenant & RBAC

Testing untuk Requirement 1: Manajemen Multi-Tenant dengan RBAC

## Flow Diagram Reference

Lihat flowchart di `sdlc/requirements.md`:
- Alur Setup Sekolah Baru (Multi-Tenant RBAC)
- Alur RBAC dan Permission Check

## Test Cases

### TC-1.1: SuperAdmin Create School

**Objective**: Verify SuperAdmin dapat membuat sekolah baru

**Prerequisites**: 
- SuperAdmin account exists
- SuperAdmin logged in

**Steps**:

1. Login sebagai SuperAdmin
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "superadmin@ppdb.com",
  "password": "admin123"
}
```

**Expected**: 
- Status: 200 OK
- Response contains `access_token`
- User role: `SuperAdmin`

2. Create new school
```http
POST /api/schools
Authorization: Bearer {superadmin_token}
Content-Type: application/json

{
  "name": "SMA Negeri 1 Jakarta",
  "npsn": "12345678",
  "address": "Jl. Sudirman No. 1, Jakarta",
  "phone": "021-1234567",
  "email": "info@sman1jakarta.sch.id",
  "logo_url": "https://example.com/logo.png"
}
```

**Expected**:
- Status: 201 Created
- Response contains school with unique `id` and `school_code`
- Status: `Active`

3. Verify school created
```http
GET /api/schools/{school_id}
Authorization: Bearer {superadmin_token}
```

**Expected**:
- Status: 200 OK
- School data matches input

---

### TC-1.2: Create School Admin

**Objective**: Verify SuperAdmin dapat membuat admin sekolah

**Steps**:

1. Create school admin user
```http
POST /api/users
Authorization: Bearer {superadmin_token}
Content-Type: application/json

{
  "email": "admin@sman1jakarta.sch.id",
  "password": "Admin123!",
  "full_name": "Admin SMA 1",
  "phone": "081234567890",
  "role": "SchoolAdmin",
  "school_id": "{school_id_from_TC1.1}"
}
```

**Expected**:
- Status: 201 Created
- User created with role `SchoolAdmin`
- User linked to school_id

2. Verify admin can login
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "admin@sman1jakarta.sch.id",
  "password": "Admin123!"
}
```

**Expected**:
- Status: 200 OK
- Token received
- User role: `SchoolAdmin`
- User school_id matches

---

### TC-1.3: Data Isolation - School Admin

**Objective**: Verify SchoolAdmin hanya bisa akses data sekolahnya

**Prerequisites**:
- 2 schools exist (School A, School B)
- Admin A logged in (school_id = A)

**Steps**:

1. Admin A list periods (should only see School A periods)
```http
GET /api/periods
Authorization: Bearer {admin_a_token}
```

**Expected**:
- Status: 200 OK
- All periods have school_id = A
- No periods from School B visible

2. Admin A try to access School B period
```http
GET /api/periods/{school_b_period_id}
Authorization: Bearer {admin_a_token}
```

**Expected**:
- Status: 404 Not Found
- Error: "Period not found" (filtered by school_id)

3. Admin A try to create period for School B
```http
POST /api/periods
Authorization: Bearer {admin_a_token}
Content-Type: application/json

{
  "school_id": "{school_b_id}",
  "name": "PPDB 2024/2025",
  "academic_year": "2024/2025",
  "level": "SMA",
  "start_date": "2024-06-01",
  "end_date": "2024-07-31"
}
```

**Expected**:
- Status: 403 Forbidden
- Error: "Access denied" or auto-override school_id to A

---

### TC-1.4: Data Isolation - Parent

**Objective**: Verify Parent hanya bisa akses data pendaftarannya sendiri

**Prerequisites**:
- Parent A logged in
- Parent A has registration R1
- Parent B has registration R2

**Steps**:

1. Parent A list own registrations
```http
GET /api/registrations
Authorization: Bearer {parent_a_token}
```

**Expected**:
- Status: 200 OK
- Only shows R1 (Parent A's registration)
- R2 not visible

2. Parent A try to access Parent B's registration
```http
GET /api/registrations/{r2_id}
Authorization: Bearer {parent_a_token}
```

**Expected**:
- Status: 404 Not Found
- Error: "Registration not found"

3. Parent A try to update Parent B's registration
```http
PUT /api/registrations/{r2_id}
Authorization: Bearer {parent_a_token}
Content-Type: application/json

{
  "student_name": "Hacked Name"
}
```

**Expected**:
- Status: 404 Not Found or 403 Forbidden

---

### TC-1.5: SuperAdmin Full Access

**Objective**: Verify SuperAdmin dapat akses semua data sekolah

**Steps**:

1. SuperAdmin list all schools
```http
GET /api/schools
Authorization: Bearer {superadmin_token}
```

**Expected**:
- Status: 200 OK
- Shows all schools (A, B, C, etc.)

2. SuperAdmin access School A data
```http
GET /api/periods?school_id={school_a_id}
Authorization: Bearer {superadmin_token}
```

**Expected**:
- Status: 200 OK
- Shows School A periods

3. SuperAdmin access School B data
```http
GET /api/periods?school_id={school_b_id}
Authorization: Bearer {superadmin_token}
```

**Expected**:
- Status: 200 OK
- Shows School B periods

---

### TC-1.6: Role-Based Permissions

**Objective**: Verify role-based access control

**Test Matrix**:

| Endpoint | SuperAdmin | SchoolAdmin | Parent | Expected |
|----------|------------|-------------|--------|----------|
| POST /api/schools | ✅ | ❌ | ❌ | 200/403/403 |
| GET /api/schools | ✅ | ❌ | ❌ | 200/403/403 |
| POST /api/periods | ✅ | ✅ | ❌ | 200/200/403 |
| GET /api/periods | ✅ | ✅ | ✅* | 200/200/200 |
| POST /api/registrations | ❌ | ❌ | ✅ | 403/403/200 |
| GET /api/verifications | ❌ | ✅ | ❌ | 403/200/403 |

*Parent can only see periods for registration

**Steps**: Test each combination above

---

### TC-1.7: School Deactivation

**Objective**: Verify deactivated school cannot be accessed

**Steps**:

1. SuperAdmin deactivate school
```http
POST /api/schools/{school_id}/deactivate
Authorization: Bearer {superadmin_token}
```

**Expected**:
- Status: 200 OK
- School status: `Inactive`

2. School Admin try to login
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "admin@deactivated-school.sch.id",
  "password": "admin123"
}
```

**Expected**:
- Status: 403 Forbidden
- Error: "School is inactive"

3. Parent try to create registration for deactivated school
```http
POST /api/registrations
Authorization: Bearer {parent_token}
Content-Type: application/json

{
  "period_id": "{deactivated_school_period_id}",
  ...
}
```

**Expected**:
- Status: 400 Bad Request
- Error: "School is not active"

---

## Test Data Setup

### SuperAdmin Account
```json
{
  "email": "superadmin@ppdb.com",
  "password": "admin123",
  "full_name": "Super Admin",
  "role": "SuperAdmin"
}
```

### Test Schools
```json
[
  {
    "name": "SMA Negeri 1 Jakarta",
    "npsn": "12345678",
    "address": "Jl. Sudirman No. 1, Jakarta",
    "phone": "021-1234567",
    "email": "info@sman1jakarta.sch.id"
  },
  {
    "name": "SMA Negeri 2 Bandung",
    "npsn": "87654321",
    "address": "Jl. Asia Afrika No. 10, Bandung",
    "phone": "022-7654321",
    "email": "info@sman2bandung.sch.id"
  }
]
```

## Validation Checklist

- [ ] SuperAdmin can create schools
- [ ] SuperAdmin can create school admins
- [ ] School admins can only access their school data
- [ ] Parents can only access their own registrations
- [ ] Data isolation works correctly
- [ ] Role-based permissions enforced
- [ ] Deactivated schools cannot be accessed
- [ ] All queries auto-filter by school_id for non-SuperAdmin

## Notes

- Semua endpoint harus auto-inject school_id filter untuk SchoolAdmin
- SuperAdmin dapat bypass school_id filter dengan query parameter
- Parent tidak perlu school_id karena filter by user_id
- Test dengan multiple schools untuk verify isolation
