# Announcement Tests Fixes

## Masalah yang Ditemukan dan Diperbaiki

### 1. Test Pattern Mismatch
**Masalah**: Test menggunakan `ctx.client` dan `ctx.base_url` yang tidak ada di TestContext

**Solusi**: Ubah test untuk menggunakan method helper yang sudah ada:
- `ctx.post()` untuk POST request
- `ctx.get()` untuk GET request

**File yang Diperbaiki**: `tests/announcement_integration_tests.rs`

---

### 2. Wrong Field Names in Response
**Masalah**: Test menggunakan field names yang tidak sesuai dengan struct response

**Error 1 - SelectionSummary**:
```rust
// ❌ Wrong
assert_eq!(response["total_registrations"].as_i64().unwrap(), 4);
assert_eq!(response["total_accepted"].as_i64().unwrap(), 3);
assert_eq!(response["total_rejected"].as_i64().unwrap(), 1);

// ✅ Correct
assert_eq!(response["verified"].as_i64().unwrap(), 0);  // After selection
assert_eq!(response["accepted"].as_i64().unwrap(), 3);
assert_eq!(response["rejected"].as_i64().unwrap(), 1);
```

**Error 2 - ResultCheckResponse**:
```rust
// ❌ Wrong
assert!(response["selection_status"].as_str().is_some());

// ✅ Correct
assert!(response["status"].as_str().is_some());
```

**File yang Diperbaiki**: `tests/announcement_integration_tests.rs`

---

### 3. Status Transition Understanding
**Masalah**: Tidak memahami perubahan status setelah run_selection

**Flow**:
1. Registration verified → status = "verified"
2. Run selection → status changes to "accepted" or "rejected"
3. After selection, verified count = 0 (all moved to accepted/rejected)

**Solusi**: Update assertion untuk reflect actual behavior:
```rust
// After run_selection, verified count becomes 0
assert_eq!(response["verified"].as_i64().unwrap(), 0);
assert_eq!(response["accepted"].as_i64().unwrap(), 3);
assert_eq!(response["rejected"].as_i64().unwrap(), 1);
```

**File yang Diperbaiki**: `tests/announcement_integration_tests.rs`

---

### 4. Helper Function Pattern
**Masalah**: Helper function perlu disesuaikan dengan pattern TestContext

**Solusi**: Rewrite helper function `setup_complete_selection_flow`:

```rust
async fn setup_complete_selection_flow(
    ctx: &TestContext,
    school_admin_token: &str,
    parent_token: &str,
    count: usize,
) -> (i32, Vec<String>) {
    // Create period with small quota
    let (_, period_response) = ctx.post(...).await;
    
    // Activate period
    ctx.post(...).await;
    
    // Create, submit, verify registrations
    for i in 0..count {
        // Create registration
        let (_, reg_response) = ctx.post(...).await;
        
        // Upload document
        ctx.post(...).await;
        
        // Submit
        let (_, submit_response) = ctx.post(...).await;
        registration_numbers.push(...);
        
        // Verify
        ctx.post(...).await;
    }
    
    // Calculate scores and rankings
    ctx.post(...).await;  // calculate-scores
    ctx.post(...).await;  // update-rankings
    
    (period_id, registration_numbers)
}
```

**File yang Diperbaiki**: `tests/announcement_integration_tests.rs`

---

## Cara Menjalankan Test

### Reset Database
```bash
sqlx database drop -y && sqlx database create && sqlx migrate run
```

### Run Individual Test
```bash
# Run selection
cargo test test_announcement_run_selection --test announcement_integration_tests -- --nocapture

# Get summary
cargo test test_announcement_get_selection_summary --test announcement_integration_tests -- --nocapture

# Announce results
cargo test test_announcement_announce_results --test announcement_integration_tests -- --nocapture

# Check result (public)
cargo test test_announcement_check_result_public --test announcement_integration_tests -- --nocapture

# Validation tests
cargo test test_announcement_check_result_invalid_nisn --test announcement_integration_tests -- --nocapture
cargo test test_announcement_check_result_not_found --test announcement_integration_tests -- --nocapture

# RBAC test
cargo test test_announcement_parent_cannot_run_selection --test announcement_integration_tests -- --nocapture
```

### Run All Announcement Tests
```bash
for test in test_announcement_run_selection test_announcement_get_selection_summary test_announcement_announce_results test_announcement_check_result_public test_announcement_check_result_invalid_nisn test_announcement_check_result_not_found test_announcement_parent_cannot_run_selection; do
    echo "Testing: $test"
    cargo test $test --test announcement_integration_tests
done
```

---

## Status Test

✅ test_announcement_run_selection
✅ test_announcement_get_selection_summary
✅ test_announcement_announce_results
✅ test_announcement_check_result_public
✅ test_announcement_check_result_invalid_nisn
✅ test_announcement_check_result_not_found
✅ test_announcement_parent_cannot_run_selection

**Total**: 7/7 tests passing

---

## Test Coverage

### 1. Run Selection
- ✅ Admin dapat menjalankan proses seleksi
- ✅ Registrations dengan ranking ≤ quota → accepted
- ✅ Registrations dengan ranking > quota → rejected
- ✅ Return total accepted dan rejected

### 2. Get Selection Summary
- ✅ Admin dapat melihat summary hasil seleksi
- ✅ Count by status (verified, accepted, rejected)
- ✅ Summary per path (quota, accepted, rejected, remaining)

### 3. Announce Results
- ✅ Admin dapat announce hasil seleksi
- ✅ Notifications sent to all participants
- ✅ Return total notified (accepted + rejected)

### 4. Check Result (Public Endpoint)
- ✅ Public dapat cek hasil dengan registration_number + NISN
- ✅ No authentication required
- ✅ Return registration details dan status

### 5. Validation Tests
- ✅ NISN must be 10 characters
- ✅ Invalid registration_number returns 404

### 6. RBAC Tests
- ✅ Parent cannot run selection
- ✅ Return 403 Forbidden

---

## Business Logic Flow

### Complete Announcement Flow
```
1. Registrations are verified (status: verified)
2. Admin calculates scores
3. Admin updates rankings
4. Admin runs selection:
   - Top N (based on quota) → accepted
   - Others → rejected
5. Admin announces results:
   - Send notifications to all
   - Email/SMS to parents
6. Public can check results:
   - Using registration_number + NISN
   - No login required
```

### Selection Logic
```rust
// For each path:
if ranking <= quota {
    status = "accepted"
} else {
    status = "rejected"
}
```

### Status Transitions
```
draft → submitted → verified → accepted/rejected
```

---

## API Endpoints

### POST /api/v1/announcements/periods/:period_id/run-selection
Run selection process for a period (admin only).

**Response**:
```json
{
  "message": "Selection completed successfully. 3 accepted, 2 rejected",
  "result": {
    "total_accepted": 3,
    "total_rejected": 2
  }
}
```

### GET /api/v1/announcements/periods/:period_id/summary
Get selection summary (admin only).

**Response**:
```json
{
  "period_id": 1,
  "verified": 0,
  "accepted": 3,
  "rejected": 2,
  "paths": [
    {
      "path_id": 1,
      "path_name": "Jalur Zonasi",
      "quota": 3,
      "accepted": 3,
      "rejected": 2,
      "remaining_quota": 0
    }
  ]
}
```

### POST /api/v1/announcements/periods/:period_id/announce
Announce results and send notifications (admin only).

**Response**:
```json
{
  "message": "Results announced successfully. 5 notifications sent (3 accepted, 2 rejected)",
  "result": {
    "total_notified": 5,
    "accepted_notified": 3,
    "rejected_notified": 2
  }
}
```

### GET /api/v1/announcements/check-result?registration_number=XXX&student_nisn=YYY
Check selection result (public endpoint, no auth required).

**Response**:
```json
{
  "registration_number": "REG-2024-001",
  "student_name": "Student Name",
  "student_nisn": "1234567890",
  "path_name": "Jalur Zonasi",
  "selection_score": 95.5,
  "ranking": 1,
  "status": "accepted",
  "rejection_reason": null,
  "announcement_date": "2024-03-15",
  "reenrollment_deadline": "2024-04-01"
}
```

---

## Catatan Penting

1. **Selection Must Be Run**: Setelah calculate scores dan update rankings, admin harus explicitly run selection untuk assign accepted/rejected status.

2. **Status Changes**: Setelah run_selection, semua verified registrations berubah menjadi accepted atau rejected. Verified count menjadi 0.

3. **Quota-Based**: Selection menggunakan quota dari path. Top N (by ranking) diterima, sisanya ditolak.

4. **Public Endpoint**: `/check-result` adalah public endpoint, tidak perlu authentication. Validasi menggunakan registration_number + NISN.

5. **NISN Validation**: NISN harus exactly 10 characters untuk security.

6. **Announcement**: Setelah run selection, admin harus announce results untuk send notifications ke parents.

7. **Test Isolation**: Setiap test menggunakan unique identifier (TEST600, TEST601, dll) untuk menghindari konflik data.

8. **Cleanup**: Selalu panggil `ctx.cleanup_test_data().await` di awal dan akhir test.

---

## Response Structs

### SelectionResult
```rust
pub struct SelectionResult {
    pub total_accepted: usize,
    pub total_rejected: usize,
}
```

### AnnouncementResult
```rust
pub struct AnnouncementResult {
    pub total_notified: usize,
    pub accepted_notified: usize,
    pub rejected_notified: usize,
}
```

### SelectionSummary
```rust
pub struct SelectionSummary {
    pub period_id: i32,
    pub verified: i64,
    pub accepted: i64,
    pub rejected: i64,
    pub paths: Vec<PathSelectionSummary>,
}
```

### ResultCheckResponse
```rust
pub struct ResultCheckResponse {
    pub registration_number: String,
    pub student_name: String,
    pub student_nisn: String,
    pub path_name: String,
    pub selection_score: Option<f64>,
    pub ranking: Option<i32>,
    pub status: String,
    pub rejection_reason: Option<String>,
    pub announcement_date: Option<NaiveDate>,
    pub reenrollment_deadline: Option<NaiveDate>,
}
```
