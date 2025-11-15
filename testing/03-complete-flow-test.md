# Complete PPDB Flow Test

Test end-to-end flow lengkap dari setup hingga enrollment sesuai requirements.

## Flow Overview

```
1. Setup (SuperAdmin)
   ↓
2. Period Setup (SchoolAdmin)
   ↓
3. Registration (Parent)
   ↓
4. Verification (SchoolAdmin)
   ↓
5. Selection (SchoolAdmin)
   ↓
6. Announcement (System)
   ↓
7. Payment (Parent)
   ↓
8. Enrollment (Parent)
```

## Complete Test Scenario

### Phase 1: Initial Setup

#### Step 1.1: SuperAdmin Creates School

```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "superadmin@ppdb.com",
  "password": "admin123"
}
```

Save `superadmin_token`

```http
POST /api/schools
Authorization: Bearer {superadmin_token}
Content-Type: application/json

{
  "name": "SMA Negeri 1 Jakarta",
  "npsn": "12345678",
  "address": "Jl. Sudirman No. 1, Jakarta Pusat",
  "phone": "021-1234567",
  "email": "info@sman1jakarta.sch.id"
}
```

Save `school_id`

#### Step 1.2: Create School Admin

```http
POST /api/users
Authorization: Bearer {superadmin_token}
Content-Type: application/json

{
  "email": "admin@sman1jakarta.sch.id",
  "password": "admin123",
  "full_name": "Admin SMA 1 Jakarta",
  "phone": "081234567890",
  "role": "SchoolAdmin",
  "school_id": {school_id}
}
```

---

### Phase 2: Period & Path Setup

#### Step 2.1: Admin Login

```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "admin@sman1jakarta.sch.id",
  "password": "admin123"
}
```

Save `admin_token`

#### Step 2.2: Create PPDB Period

```http
POST /api/periods
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "name": "PPDB 2024/2025",
  "academic_year": "2024/2025",
  "level": "SMA",
  "start_date": "2024-06-01T00:00:00Z",
  "end_date": "2024-07-31T23:59:59Z",
  "registration_start": "2024-06-01T00:00:00Z",
  "registration_end": "2024-06-30T23:59:59Z",
  "announcement_date": "2024-07-15T00:00:00Z"
}
```

Save `period_id`

#### Step 2.3: Create Registration Paths

**Jalur Zonasi**
```http
POST /api/periods/{period_id}/paths
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "name": "Jalur Zonasi",
  "path_type": "Zonasi",
  "quota": 180,
  "description": "Jalur zonasi berdasarkan jarak tempat tinggal"
}
```

Save `zonasi_path_id`

**Jalur Prestasi**
```http
POST /api/periods/{period_id}/paths
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "name": "Jalur Prestasi",
  "path_type": "Prestasi",
  "quota": 90,
  "description": "Jalur prestasi akademik dan non-akademik"
}
```

Save `prestasi_path_id`

**Jalur Afirmasi**
```http
POST /api/periods/{period_id}/paths
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "name": "Jalur Afirmasi",
  "path_type": "Afirmasi",
  "quota": 20,
  "description": "Jalur afirmasi untuk siswa kurang mampu"
}
```

Save `afirmasi_path_id`

**Jalur Perpindahan Tugas**
```http
POST /api/periods/{period_id}/paths
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "name": "Jalur Perpindahan Tugas Orang Tua",
  "path_type": "PerpindahanTugas",
  "quota": 10,
  "description": "Jalur perpindahan tugas orang tua/wali"
}
```

Save `perpindahan_path_id`

#### Step 2.4: Activate Period

```http
POST /api/periods/{period_id}/activate
Authorization: Bearer {admin_token}
```

---

### Phase 3: Parent Registration

#### Step 3.1: Parent Register & Login

```http
POST /api/auth/register
Content-Type: application/json

{
  "email": "parent1@example.com",
  "password": "admin123",
  "full_name": "Budi Santoso",
  "phone": "081111111111",
  "nik": "3201010101010001"
}
```

```http
POST /api/auth/verify-email
Content-Type: application/json

{
  "token": "{verification_token}"
}
```

```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "parent1@example.com",
  "password": "admin123"
}
```

Save `parent_token`

#### Step 3.2: Create Registration (Draft)

```http
POST /api/registrations
Authorization: Bearer {parent_token}
Content-Type: application/json

{
  "period_id": {period_id},
  "path_id": {zonasi_path_id},
  "student_nisn": "0012345678",
  "student_nik": "3201234567890124",
  "student_name": "Ahmad Santoso",
  "student_birth_place": "Jakarta",
  "student_birth_date": "2010-05-15",
  "student_gender": "Male",
  "student_address": "Jl. Kebon Jeruk No. 10, Jakarta Barat",
  "previous_school": "SMP Negeri 5 Jakarta",
  "distance_km": 2.5,
  "avg_report_score": 85.5
}
```

Save `registration_id`

#### Step 3.3: Upload Documents

**Upload Kartu Keluarga**
```http
POST /api/registrations/{registration_id}/documents
Authorization: Bearer {parent_token}
Content-Type: multipart/form-data

{
  "document_type": "KartuKeluarga",
  "file": {kk.pdf}
}
```

**Upload Akta Kelahiran**
```http
POST /api/registrations/{registration_id}/documents
Authorization: Bearer {parent_token}
Content-Type: multipart/form-data

{
  "document_type": "AktaKelahiran",
  "file": {akta.pdf}
}
```

**Upload Rapor**
```http
POST /api/registrations/{registration_id}/documents
Authorization: Bearer {parent_token}
Content-Type: multipart/form-data

{
  "document_type": "Rapor",
  "file": {rapor.pdf}
}
```

#### Step 3.4: Submit Registration

```http
POST /api/registrations/{registration_id}/submit
Authorization: Bearer {parent_token}
```

**Expected**: Status changes to `Submitted`

---

### Phase 4: Verification by Admin

#### Step 4.1: View Pending Verifications

```http
GET /api/verifications/pending
Authorization: Bearer {admin_token}
```

#### Step 4.2: View Registration Detail

```http
GET /api/registrations/{registration_id}
Authorization: Bearer {admin_token}
```

#### Step 4.3: Verify Documents

```http
POST /api/verifications/documents/{document_id}/verify
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "status": "Verified",
  "notes": "Dokumen valid dan sesuai"
}
```

Repeat for all documents

#### Step 4.4: Approve Registration

```http
POST /api/verifications/registrations/{registration_id}/verify
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "notes": "Semua dokumen lengkap dan valid"
}
```

**Expected**: 
- Status changes to `Verified`
- Score automatically calculated
- Parent receives email notification

---

### Phase 5: Selection Process

#### Step 5.1: Calculate All Scores

```http
POST /api/selection/calculate-scores
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "period_id": {period_id}
}
```

#### Step 5.2: Update Rankings

```http
POST /api/selection/update-rankings
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "period_id": {period_id}
}
```

#### Step 5.3: View Rankings

```http
GET /api/selection/rankings?period_id={period_id}&path_id={zonasi_path_id}
Authorization: Bearer {admin_token}
```

#### Step 5.4: Run Selection

```http
POST /api/announcements/run-selection
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "period_id": {period_id}
}
```

**Expected**:
- Top N students (by quota) marked as `Accepted`
- Others marked as `Rejected`
- Selection summary returned

---

### Phase 6: Announcement

#### Step 6.1: Announce Results

```http
POST /api/announcements/announce
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "period_id": {period_id}
}
```

**Expected**:
- All students receive email notifications
- Results published

#### Step 6.2: Parent Check Result

```http
GET /api/announcements/check-result?registration_number={reg_number}&nisn={nisn}
```

**Expected**:
- Shows acceptance status
- If accepted: shows payment info
- If rejected: shows reason

---

### Phase 7: Payment (For Accepted Students)

#### Step 7.1: View Payment Info

```http
GET /api/registrations/{registration_id}
Authorization: Bearer {parent_token}
```

**Expected**: Shows payment details and amount

#### Step 7.2: Generate Virtual Account

```http
POST /api/payments/generate-va
Authorization: Bearer {parent_token}
Content-Type: application/json

{
  "registration_id": {registration_id},
  "payment_method": "VirtualAccount"
}
```

**Expected**: Returns VA number

#### Step 7.3: Simulate Payment (via webhook)

```http
POST /api/webhooks/payment
Content-Type: application/json
X-Signature: {webhook_signature}

{
  "va_number": "{va_number}",
  "amount": 500000,
  "status": "paid",
  "transaction_id": "TRX123456789"
}
```

**Expected**:
- Payment status updated to `Paid`
- Parent receives confirmation email

---

### Phase 8: Enrollment

#### Step 8.1: Confirm Enrollment

```http
POST /api/registrations/{registration_id}/enroll
Authorization: Bearer {parent_token}
Content-Type: application/json

{
  "confirmation": true
}
```

**Expected**:
- Status changes to `Enrolled`
- Enrollment confirmation sent
- Student data ready for sync

#### Step 8.2: View Final Registration Proof

```http
GET /api/registrations/{registration_id}/proof
Authorization: Bearer {parent_token}
```

**Expected**: Returns PDF proof of enrollment

---

## Test Multiple Students

Repeat Phase 3-8 for multiple students:

### Student Profiles

**Student 1 - Zonasi (Accepted)**
- Distance: 2.5 km
- Score: 85.5
- Expected: Accepted (within quota)

**Student 2 - Prestasi (Accepted)**
- Avg Score: 92.0
- Achievements: 3 certificates
- Expected: Accepted (high score)

**Student 3 - Zonasi (Rejected)**
- Distance: 8.0 km
- Score: 75.0
- Expected: Rejected (outside quota)

**Student 4 - Afirmasi (Accepted)**
- KIP holder
- Score: 80.0
- Expected: Accepted (afirmasi quota)

---

## Validation Checklist

### Setup Phase
- [ ] School created successfully
- [ ] Admin account created and linked to school
- [ ] Admin can login

### Period Phase
- [ ] Period created with correct dates
- [ ] All 4 paths created with quotas
- [ ] Period activated successfully

### Registration Phase
- [ ] Parent can register and verify email
- [ ] Registration created as draft
- [ ] Documents uploaded successfully
- [ ] Registration submitted

### Verification Phase
- [ ] Admin can see pending verifications
- [ ] Documents can be verified individually
- [ ] Registration approved
- [ ] Score calculated automatically

### Selection Phase
- [ ] Scores calculated for all verified
- [ ] Rankings updated correctly
- [ ] Selection run successfully
- [ ] Correct students accepted/rejected based on quota

### Announcement Phase
- [ ] Results announced
- [ ] Email notifications sent
- [ ] Public result check works

### Payment Phase
- [ ] Payment info displayed
- [ ] VA generated
- [ ] Payment webhook processed
- [ ] Payment confirmed

### Enrollment Phase
- [ ] Enrollment confirmed
- [ ] Final proof generated
- [ ] Status updated to Enrolled

## Expected Results Summary

| Student | Path | Score | Rank | Quota | Result |
|---------|------|-------|------|-------|--------|
| Ahmad | Zonasi | 95.0 | 1 | 180 | ✅ Accepted |
| Budi | Prestasi | 92.0 | 1 | 90 | ✅ Accepted |
| Citra | Afirmasi | 80.0 | 1 | 20 | ✅ Accepted |
| Dedi | Zonasi | 65.0 | 200 | 180 | ❌ Rejected |

## Notes

- Test dengan data realistis
- Verify email notifications sent
- Check audit logs
- Verify data isolation between schools
- Test error scenarios (rejected, expired, etc.)
- Verify payment integration
- Check SSO sync (if implemented)
