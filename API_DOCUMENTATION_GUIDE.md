# API Documentation Guide

## Overview

PPDB Backend API menggunakan **OpenAPI 3.0** (Swagger) untuk dokumentasi API yang interaktif dan auto-generated.

## Technology Stack

- **utoipa** - OpenAPI documentation generator untuk Rust
- **utoipa-swagger-ui** - Swagger UI (interactive API testing)
- **utoipa-rapidoc** - RapiDoc UI (alternative modern UI)
- **utoipa-redoc** - ReDoc UI (clean documentation view)

## Accessing Documentation

Setelah server berjalan, akses dokumentasi di:

### 🎨 Swagger UI (Recommended)
```
http://localhost:8000/api/docs/swagger
```
- Interactive API testing
- Try out endpoints directly
- See request/response examples

### 🚀 RapiDoc
```
http://localhost:8000/api/docs/rapidoc
```
- Modern, fast UI
- Better for mobile
- Dark mode support

### 📚 ReDoc
```
http://localhost:8000/api/docs/redoc
```
- Clean, professional look
- Best for reading documentation
- Print-friendly

### 📄 OpenAPI Spec (JSON)
```
http://localhost:8000/api/docs/openapi.json
```
- Raw OpenAPI specification
- Import to Postman/Insomnia
- Generate API clients

## How to Document Endpoints

### Step 1: Add `#[utoipa::path]` macro to handler

```rust
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "user@example.com")]
    pub email: String,
    
    #[schema(example = "password123")]
    pub password: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// User login endpoint
/// 
/// Authenticates user with email and password, returns JWT token.
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ValidationErrorResponse)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    // Implementation
}
```

### Step 2: Add DTOs with `#[derive(ToSchema)]`

```rust
use utoipa::ToSchema;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "SMA Negeri 1 Jakarta",
    "npsn": "12345678",
    "address": "Jl. Sudirman No. 1"
}))]
pub struct CreateSchoolDto {
    /// School name
    #[schema(example = "SMA Negeri 1 Jakarta")]
    pub name: String,
    
    /// NPSN (Nomor Pokok Sekolah Nasional)
    #[schema(example = "12345678", min_length = 8, max_length = 8)]
    pub npsn: String,
    
    /// School address
    #[schema(example = "Jl. Sudirman No. 1")]
    pub address: String,
    
    /// Contact phone
    #[schema(example = "+62211234567")]
    pub phone: Option<String>,
    
    /// Contact email
    #[schema(example = "info@sman1jakarta.sch.id", format = "email")]
    pub email: Option<String>,
}
```

### Step 3: Add enums with `#[derive(ToSchema)]`

```rust
use utoipa::ToSchema;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    #[schema(rename = "super_admin")]
    SuperAdmin,
    
    #[schema(rename = "school_admin")]
    SchoolAdmin,
    
    #[schema(rename = "parent")]
    Parent,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationStatus {
    Draft,
    Submitted,
    Verified,
    Rejected,
    Accepted,
    Enrolled,
    Expired,
}
```

### Step 4: Register in ApiDoc

Edit `src/api/docs.rs`:

```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        // Add your endpoint here
        crate::api::auth::login,
        crate::api::auth::register,
        // ...
    ),
    components(
        schemas(
            // Add your DTOs here
            crate::dto::auth_dto::LoginRequest,
            crate::dto::auth_dto::LoginResponse,
            // ...
        )
    ),
    tags(
        (name = "Authentication", description = "User authentication endpoints"),
        // ...
    )
)]
pub struct ApiDoc;
```

## Advanced Features

### Adding Security Requirements

For endpoints that require authentication:

```rust
#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    tag = "Users",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "User profile", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
pub async fn get_current_user(
    auth_user: AuthUser,
) -> AppResult<Json<UserResponse>> {
    // Implementation
}
```

### Adding Query Parameters

```rust
#[derive(Deserialize, IntoParams)]
pub struct ListQuery {
    /// Page number (starts from 1)
    #[param(example = 1, minimum = 1)]
    pub page: Option<i64>,
    
    /// Items per page
    #[param(example = 20, minimum = 1, maximum = 100)]
    pub per_page: Option<i64>,
    
    /// Search query
    #[param(example = "Jakarta")]
    pub search: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/schools",
    tag = "Schools",
    params(ListQuery),
    responses(
        (status = 200, description = "List of schools", body = Vec<SchoolResponse>)
    )
)]
pub async fn list_schools(
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<SchoolResponse>>> {
    // Implementation
}
```

### Adding Path Parameters

```rust
#[utoipa::path(
    get,
    path = "/api/v1/schools/{id}",
    tag = "Schools",
    params(
        ("id" = Uuid, Path, description = "School ID", example = "550e8400-e29b-41d4-a716-446655440000")
    ),
    responses(
        (status = 200, description = "School details", body = SchoolResponse),
        (status = 404, description = "School not found", body = ErrorResponse)
    )
)]
pub async fn get_school(
    Path(id): Path<Uuid>,
) -> AppResult<Json<SchoolResponse>> {
    // Implementation
}
```

### Adding File Upload Documentation

```rust
#[utoipa::path(
    post,
    path = "/api/v1/registrations/{id}/documents",
    tag = "Registrations",
    params(
        ("id" = Uuid, Path, description = "Registration ID")
    ),
    request_body(
        content = inline(UploadDocumentRequest),
        content_type = "multipart/form-data"
    ),
    responses(
        (status = 201, description = "Document uploaded", body = DocumentResponse),
        (status = 400, description = "Invalid file", body = ErrorResponse)
    )
)]
pub async fn upload_document(
    Path(id): Path<Uuid>,
    multipart: Multipart,
) -> AppResult<Json<DocumentResponse>> {
    // Implementation
}
```

## Best Practices

### 1. Always Add Examples

```rust
#[derive(ToSchema)]
pub struct School {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    
    #[schema(example = "SMA Negeri 1 Jakarta")]
    pub name: String,
    
    #[schema(example = "12345678")]
    pub npsn: String,
}
```

### 2. Add Field Descriptions

```rust
#[derive(ToSchema)]
pub struct CreateRegistrationDto {
    /// Student's NISN (Nomor Induk Siswa Nasional)
    #[schema(example = "1234567890", min_length = 10, max_length = 10)]
    pub student_nisn: String,
    
    /// Student's full name as in birth certificate
    #[schema(example = "Ahmad Rizki Pratama")]
    pub student_name: String,
}
```

### 3. Document All Response Codes

```rust
#[utoipa::path(
    post,
    path = "/api/v1/registrations",
    responses(
        (status = 201, description = "Registration created", body = RegistrationResponse),
        (status = 400, description = "Invalid input", body = ValidationErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Period not active", body = ErrorResponse),
        (status = 404, description = "Period not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
```

### 4. Group Related Endpoints with Tags

```rust
#[utoipa::path(
    tag = "Registrations",
    // ...
)]
```

### 5. Add Detailed Descriptions

```rust
/// Create new student registration
/// 
/// Creates a new registration in draft status. The registration can be edited
/// until it is submitted. Once submitted, it cannot be modified.
/// 
/// # Requirements
/// - Period must be active
/// - User must be authenticated as Parent
/// - Path quota must not be full
/// 
/// # Business Rules
/// - Registration starts in Draft status
/// - Registration number is generated on submit
/// - Documents can be uploaded after creation
#[utoipa::path(
    // ...
)]
```

## Testing Documentation

### 1. Check Compilation

```bash
cargo check
```

### 2. Run Server

```bash
cargo run
```

### 3. Access Swagger UI

Open browser: `http://localhost:8000/api/docs/swagger`

### 4. Test Endpoints

1. Click "Authorize" button
2. Enter JWT token: `Bearer <your_token>`
3. Try out endpoints directly from UI

## Generating API Clients

### Export OpenAPI Spec

```bash
curl http://localhost:8000/api/docs/openapi.json > openapi.json
```

### Generate TypeScript Client

```bash
npx @openapitools/openapi-generator-cli generate \
  -i openapi.json \
  -g typescript-axios \
  -o ./client/typescript
```

### Generate Python Client

```bash
openapi-generator-cli generate \
  -i openapi.json \
  -g python \
  -o ./client/python
```

### Import to Postman

1. Open Postman
2. Import → Link
3. Enter: `http://localhost:8000/api/docs/openapi.json`
4. All endpoints will be imported with examples

## Troubleshooting

### Error: "cannot find macro `utoipa` in this scope"

Add to Cargo.toml:
```toml
utoipa = { version = "4.2", features = ["axum_extras", "chrono", "uuid"] }
```

### Error: "trait `ToSchema` is not implemented"

Add derive macro:
```rust
#[derive(utoipa::ToSchema)]
pub struct YourStruct { }
```

### Documentation Not Updating

1. Clean build: `cargo clean`
2. Rebuild: `cargo build`
3. Restart server: `cargo run`

### Missing Endpoints in Swagger

Check that endpoint is registered in `ApiDoc`:
```rust
#[openapi(
    paths(
        crate::api::your_module::your_endpoint,  // Add here
    )
)]
```

## Resources

- [utoipa Documentation](https://docs.rs/utoipa/)
- [OpenAPI Specification](https://swagger.io/specification/)
- [Swagger UI](https://swagger.io/tools/swagger-ui/)
- [RapiDoc](https://rapidocweb.com/)
- [ReDoc](https://redocly.com/)

## Next Steps

1. ✅ Setup dependencies (Done)
2. ✅ Create docs module (Done)
3. ✅ Add Swagger UI routes (Done)
4. ⏳ Document all auth endpoints
5. ⏳ Document all school endpoints
6. ⏳ Document all user endpoints
7. ⏳ Document all period endpoints
8. ⏳ Document all registration endpoints
9. ⏳ Document all selection endpoints
10. ⏳ Add comprehensive examples
11. ⏳ Generate OpenAPI spec file
12. ⏳ Create Postman collection

## Estimated Timeline

- Phase 1: Setup (Done) ✅
- Phase 2: Document core endpoints (2-3 days)
- Phase 3: Add examples and descriptions (2-3 days)
- Phase 4: Testing and refinement (1-2 days)

**Total: 1-2 weeks for complete documentation**
