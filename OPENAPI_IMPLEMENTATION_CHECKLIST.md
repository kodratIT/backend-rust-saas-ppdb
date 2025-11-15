# OpenAPI Implementation Checklist

## Progress Tracker

### ✅ Phase 1: Foundation (COMPLETE)
- [x] Setup utoipa dependencies
- [x] Create docs module
- [x] Add Swagger UI routes
- [x] Create example endpoint (health check)
- [x] Create documentation guides

### ⏳ Phase 2: Document DTOs (IN PROGRESS)

#### Authentication DTOs
- [x] RegisterRequest
- [x] LoginRequest
- [x] AuthResponse
- [x] RefreshTokenRequest
- [x] RefreshTokenResponse
- [x] UserResponse
- [x] VerifyEmailRequest
- [x] ForgotPasswordRequest
- [x] ResetPasswordRequest

#### School DTOs
- [ ] CreateSchoolDto
- [ ] UpdateSchoolDto
- [ ] SchoolResponse

#### User DTOs
- [ ] CreateUserDto
- [ ] UpdateUserDto
- [ ] UserResponse (already done in auth)

#### Period DTOs
- [ ] CreatePeriodDto
- [ ] UpdatePeriodDto
- [ ] PeriodResponse
- [ ] CreatePathDto
- [ ] UpdatePathDto
- [ ] PathResponse

#### Registration DTOs
- [ ] CreateRegistrationDto
- [ ] UpdateRegistrationDto
- [ ] RegistrationResponse
- [ ] VerifyRegistrationDto
- [ ] RejectRegistrationDto

#### Document DTOs
- [ ] DocumentResponse
- [ ] UploadDocumentRequest

#### Selection DTOs
- [ ] CalculateScoresRequest
- [ ] RankingResponse
- [ ] SelectionSummaryResponse
- [ ] CheckResultRequest
- [ ] CheckResultResponse

#### Common Enums
- [ ] UserRole
- [ ] SchoolStatus
- [ ] PeriodStatus
- [ ] Level
- [ ] PathType
- [ ] RegistrationStatus
- [ ] DocumentType
- [ ] VerificationStatus

### ⏳ Phase 3: Document Endpoints

#### System Endpoints
- [x] GET /health - Health check

#### Authentication Endpoints
- [ ] POST /api/v1/auth/register
- [ ] POST /api/v1/auth/login
- [ ] POST /api/v1/auth/verify-email
- [ ] POST /api/v1/auth/forgot-password
- [ ] POST /api/v1/auth/reset-password
- [ ] POST /api/v1/auth/refresh
- [ ] POST /api/v1/auth/logout
- [ ] GET /api/v1/auth/me

#### School Endpoints
- [ ] GET /api/v1/schools
- [ ] POST /api/v1/schools
- [ ] GET /api/v1/schools/{id}
- [ ] PUT /api/v1/schools/{id}
- [ ] DELETE /api/v1/schools/{id}

#### User Endpoints
- [ ] GET /api/v1/users
- [ ] POST /api/v1/users
- [ ] GET /api/v1/users/{id}
- [ ] PUT /api/v1/users/{id}
- [ ] DELETE /api/v1/users/{id}
- [ ] GET /api/v1/users/me
- [ ] PUT /api/v1/users/me

#### Period Endpoints
- [ ] GET /api/v1/periods
- [ ] POST /api/v1/periods
- [ ] GET /api/v1/periods/{id}
- [ ] PUT /api/v1/periods/{id}
- [ ] DELETE /api/v1/periods/{id}
- [ ] POST /api/v1/periods/{id}/activate
- [ ] POST /api/v1/periods/{id}/close

#### Registration Endpoints
- [ ] GET /api/v1/registrations
- [ ] POST /api/v1/registrations
- [ ] GET /api/v1/registrations/{id}
- [ ] PUT /api/v1/registrations/{id}
- [ ] POST /api/v1/registrations/{id}/submit
- [ ] POST /api/v1/registrations/{id}/verify
- [ ] POST /api/v1/registrations/{id}/reject
- [ ] GET /api/v1/registrations/pending

#### Document Endpoints
- [ ] GET /api/v1/registrations/{id}/documents
- [ ] POST /api/v1/registrations/{id}/documents
- [ ] DELETE /api/v1/registrations/{id}/documents/{doc_id}

#### Selection Endpoints
- [ ] POST /api/v1/periods/{id}/calculate-scores
- [ ] GET /api/v1/periods/{id}/rankings
- [ ] POST /api/v1/periods/{id}/run-selection
- [ ] POST /api/v1/periods/{id}/announce
- [ ] GET /api/v1/selection/result

### ⏳ Phase 4: Register in ApiDoc

- [ ] Register all paths in docs.rs
- [ ] Register all schemas in docs.rs
- [ ] Verify compilation
- [ ] Test in Swagger UI

### ⏳ Phase 5: Testing & Refinement

- [ ] Test all endpoints in Swagger UI
- [ ] Verify authentication flow
- [ ] Test with real data
- [ ] Fix any issues
- [ ] Add missing examples
- [ ] Improve descriptions

### ⏳ Phase 6: Export & Distribution

- [ ] Generate openapi.json
- [ ] Generate openapi.yaml
- [ ] Create Postman collection
- [ ] Test Postman import
- [ ] Generate TypeScript client
- [ ] Generate Python client
- [ ] Update README with links

---

## Quick Commands

### Check compilation
```bash
cd ppdb-sekolah/backend
cargo check
```

### Run server
```bash
cargo run
```

### Access Swagger UI
```
http://localhost:8000/api/docs/swagger
```

### Export OpenAPI spec
```bash
curl http://localhost:8000/api/docs/openapi.json > openapi.json
```

---

## Current Status

**Phase:** 2 - Document DTOs  
**Progress:** 9/50+ DTOs documented (18%)  
**Next Task:** Document School DTOs  
**Estimated Time Remaining:** 4-6 days

---

## Notes

- Focus on one module at a time (Auth → Schools → Users → Periods → Registrations → Selection)
- Test each module in Swagger UI before moving to next
- Add comprehensive examples for complex DTOs
- Document all business rules in endpoint descriptions
- Keep error responses consistent across endpoints

---

## Resources

- [utoipa Documentation](https://docs.rs/utoipa/)
- [OpenAPI Specification](https://swagger.io/specification/)
- [OPENAPI_EXAMPLES.md](./OPENAPI_EXAMPLES.md) - Code examples
- [API_DOCUMENTATION_GUIDE.md](./API_DOCUMENTATION_GUIDE.md) - Detailed guide
