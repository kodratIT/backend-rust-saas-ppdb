# Test Scenario 2: Authentication Flow

Testing untuk Requirement 2: Manajemen Akun Pengguna

## Flow Diagram Reference

Lihat flowchart di `sdlc/requirements.md`:
- Alur Pengguna (Orang Tua/Wali) - bagian registrasi dan login

## Test Cases

### TC-2.1: User Registration (Parent)

**Objective**: Verify orang tua dapat mendaftar akun baru

**Steps**:

1. Register new parent account
```http
POST /api/auth/register
Content-Type: application/json

{
  "email": "parent1@example.com",
  "password": "Parent123!",
  "full_name": "Budi Santoso",
  "phone": "081234567890",
  "nik": "3201234567890123"
}
```

**Expected**:
- Status: 201 Created
- Response contains user data (without password)
- Email verification sent
- User status: `Unverified`

**Validation**:
- Email format valid
- Password min 8 chars, contains uppercase, lowercase, number
- Phone format valid (Indonesian)
- NIK 16 digits

2. Try register with duplicate email
```http
POST /api/auth/register
Content-Type: application/json

{
  "email": "parent1@example.com",
  "password": "Parent123!",
  "full_name": "Another User",
  "phone": "081234567891",
  "nik": "3201234567890124"
}
```

**Expected**:
- Status: 400 Bad Request
- Error: "Email already registered"

---

### TC-2.2: Email Verification

**Objective**: Verify email verification flow

**Steps**:

1. Get verification token from email (simulated)
```
Token format: base64 encoded string
Example: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

2. Verify email
```http
POST /api/auth/verify-email
Content-Type: application/json

{
  "token": "{verification_token}"
}
```

**Expected**:
- Status: 200 OK
- Message: "Email verified successfully"
- User status updated to `Active`

3. Try verify with invalid token
```http
POST /api/auth/verify-email
Content-Type: application/json

{
  "token": "invalid_token_123"
}
```

**Expected**:
- Status: 400 Bad Request
- Error: "Invalid or expired verification token"

4. Try verify already verified email
```http
POST /api/auth/verify-email
Content-Type: application/json

{
  "token": "{already_used_token}"
}
```

**Expected**:
- Status: 400 Bad Request
- Error: "Email already verified"

---

### TC-2.3: Login

**Objective**: Verify user login flow

**Steps**:

1. Login with valid credentials
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "parent1@example.com",
  "password": "Parent123!"
}
```

**Expected**:
- Status: 200 OK
- Response contains:
  - `access_token` (JWT)
  - `refresh_token`
  - `token_type`: "Bearer"
  - `expires_in`: 86400 (24 hours)
  - `user` object with role, email, name

2. Login with wrong password
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "parent1@example.com",
  "password": "WrongPassword123!"
}
```

**Expected**:
- Status: 401 Unauthorized
- Error: "Invalid credentials"

3. Login with non-existent email
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "notexist@example.com",
  "password": "Password123!"
}
```

**Expected**:
- Status: 401 Unauthorized
- Error: "Invalid credentials"

4. Login with unverified email
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "unverified@example.com",
  "password": "Password123!"
}
```

**Expected**:
- Status: 403 Forbidden
- Error: "Email not verified. Please check your email."

---

### TC-2.4: Get Current User

**Objective**: Verify authenticated user can get their profile

**Prerequisites**: User logged in

**Steps**:

1. Get current user profile
```http
GET /api/auth/me
Authorization: Bearer {access_token}
```

**Expected**:
- Status: 200 OK
- Response contains user data:
  - id, email, full_name, phone, role
  - school_id (if SchoolAdmin)
  - No password field

2. Try without token
```http
GET /api/auth/me
```

**Expected**:
- Status: 401 Unauthorized
- Error: "Missing authorization token"

3. Try with invalid token
```http
GET /api/auth/me
Authorization: Bearer invalid_token_123
```

**Expected**:
- Status: 401 Unauthorized
- Error: "Invalid token"

---

### TC-2.5: Refresh Token

**Objective**: Verify token refresh mechanism

**Prerequisites**: User logged in with refresh_token

**Steps**:

1. Refresh access token
```http
POST /api/auth/refresh
Content-Type: application/json

{
  "refresh_token": "{refresh_token_from_login}"
}
```

**Expected**:
- Status: 200 OK
- Response contains new `access_token`
- New token has extended expiry

2. Try with invalid refresh token
```http
POST /api/auth/refresh
Content-Type: application/json

{
  "refresh_token": "invalid_refresh_token"
}
```

**Expected**:
- Status: 401 Unauthorized
- Error: "Invalid refresh token"

3. Try with expired refresh token
```http
POST /api/auth/refresh
Content-Type: application/json

{
  "refresh_token": "{expired_refresh_token}"
}
```

**Expected**:
- Status: 401 Unauthorized
- Error: "Refresh token expired"

---

### TC-2.6: Forgot Password

**Objective**: Verify password reset request flow

**Steps**:

1. Request password reset
```http
POST /api/auth/forgot-password
Content-Type: application/json

{
  "email": "parent1@example.com"
}
```

**Expected**:
- Status: 200 OK
- Message: "Password reset email sent"
- Reset token sent to email

2. Request for non-existent email
```http
POST /api/auth/forgot-password
Content-Type: application/json

{
  "email": "notexist@example.com"
}
```

**Expected**:
- Status: 200 OK (security: don't reveal if email exists)
- Message: "If email exists, reset link will be sent"

---

### TC-2.7: Reset Password

**Objective**: Verify password reset flow

**Prerequisites**: Reset token received from forgot password

**Steps**:

1. Reset password with valid token
```http
POST /api/auth/reset-password
Content-Type: application/json

{
  "token": "{reset_token}",
  "new_password": "NewPassword123!"
}
```

**Expected**:
- Status: 200 OK
- Message: "Password reset successfully"
- Old password no longer works
- Can login with new password

2. Try with invalid token
```http
POST /api/auth/reset-password
Content-Type: application/json

{
  "token": "invalid_token",
  "new_password": "NewPassword123!"
}
```

**Expected**:
- Status: 400 Bad Request
- Error: "Invalid or expired reset token"

3. Try with weak password
```http
POST /api/auth/reset-password
Content-Type: application/json

{
  "token": "{reset_token}",
  "new_password": "weak"
}
```

**Expected**:
- Status: 400 Bad Request
- Error: "Password must be at least 8 characters..."

---

### TC-2.8: Logout

**Objective**: Verify logout invalidates token

**Prerequisites**: User logged in

**Steps**:

1. Logout
```http
POST /api/auth/logout
Authorization: Bearer {access_token}
```

**Expected**:
- Status: 200 OK
- Message: "Logged out successfully"
- Token added to blacklist

2. Try to use logged out token
```http
GET /api/auth/me
Authorization: Bearer {logged_out_token}
```

**Expected**:
- Status: 401 Unauthorized
- Error: "Token has been revoked"

---

### TC-2.9: Password Validation

**Objective**: Verify password requirements

**Test Cases**:

| Password | Expected Result | Reason |
|----------|----------------|---------|
| `Pass123!` | ✅ Valid | Meets all requirements |
| `pass123!` | ❌ Invalid | No uppercase |
| `PASS123!` | ❌ Invalid | No lowercase |
| `Password!` | ❌ Invalid | No number |
| `Password123` | ❌ Invalid | No special char |
| `Pass1!` | ❌ Invalid | Too short (<8) |
| `Pass 123!` | ❌ Invalid | Contains space |

---

### TC-2.10: Rate Limiting

**Objective**: Verify rate limiting on login attempts

**Steps**:

1. Make 5 failed login attempts rapidly
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "parent1@example.com",
  "password": "WrongPassword"
}
```

Repeat 5 times quickly

**Expected**:
- First 3 attempts: 401 Unauthorized
- 4th attempt onwards: 429 Too Many Requests
- Error: "Too many login attempts. Try again in X minutes"

2. Wait for cooldown period (e.g., 15 minutes)

3. Try login again
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "parent1@example.com",
  "password": "Parent123!"
}
```

**Expected**:
- Status: 200 OK
- Login successful

---

## Test Data

### Valid Test Users

```json
[
  {
    "email": "parent1@example.com",
    "password": "admin123",
    "full_name": "Budi Santoso",
    "phone": "081111111111",
    "nik": "3201010101010001",
    "role": "Parent"
  },
  {
    "email": "admin@sman1jkt.sch.id",
    "password": "admin123",
    "full_name": "Admin SMA 1 Jakarta",
    "phone": "081234567890",
    "role": "SchoolAdmin",
    "school_id": 1
  },
  {
    "email": "superadmin@ppdb.com",
    "password": "admin123",
    "full_name": "Super Admin",
    "role": "SuperAdmin"
  }
]
```

### Invalid Test Cases

```json
{
  "invalid_emails": [
    "notanemail",
    "@example.com",
    "user@",
    "user @example.com"
  ],
  "weak_passwords": [
    "12345678",
    "password",
    "Password",
    "Pass123"
  ]
}
```

## Validation Checklist

- [ ] Registration with valid data succeeds
- [ ] Duplicate email rejected
- [ ] Email verification works
- [ ] Login with valid credentials succeeds
- [ ] Login with invalid credentials fails
- [ ] Unverified users cannot login
- [ ] JWT token contains correct claims
- [ ] Token refresh works
- [ ] Forgot password sends email
- [ ] Reset password works with valid token
- [ ] Logout invalidates token
- [ ] Password validation enforced
- [ ] Rate limiting prevents brute force
- [ ] All sensitive data encrypted

## Security Notes

- Passwords hashed with bcrypt/argon2
- JWT tokens expire in 24 hours
- Refresh tokens expire in 7 days
- Rate limiting: 5 attempts per 15 minutes
- Email verification required before login
- Password reset tokens expire in 1 hour
- All tokens stored securely (not in localStorage for production)
