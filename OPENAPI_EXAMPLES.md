# OpenAPI Documentation Examples

This file contains examples of how to document endpoints with utoipa.

## Example 1: Simple POST endpoint with authentication

```rust
use utoipa::ToSchema;

/// User login endpoint
/// 
/// Authenticates user with email and password, returns JWT tokens.
/// 
/// # Authentication
/// This endpoint does not require authentication.
/// 
/// # Returns
/// - `200 OK`: Login successful, returns access token and user info
/// - `401 Unauthorized`: Invalid credentials
/// - `422 Unprocessable Entity`: Validation error
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse,
            example = json!({
                "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
                "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
                "token_type": "Bearer",
                "expires_in": 86400,
                "user": {
                    "id": 1,
                    "email": "admin@example.com",
                    "full_name": "Admin User",
                    "role": "school_admin",
                    "school_id": 1
                }
            })
        ),
        (status = 401, description = "Invalid credentials", body = ErrorResponse,
            example = json!({
                "error": "Invalid email or password"
            })
        ),
        (status = 422, description = "Validation error", body = ValidationErrorResponse,
            example = json!({
                "error": "Validation failed",
                "fields": [
                    {
                        "field": "email",
                        "message": "Invalid email format"
                    }
                ]
            })
        )
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    // Implementation
}
```

## Example 2: GET endpoint with authentication and query parameters

```rust
/// List all schools
/// 
/// Returns a paginated list of all schools in the system.
/// 
/// # Authentication
/// Requires JWT Bearer token with SuperAdmin role.
/// 
/// # Query Parameters
/// - `page`: Page number (default: 1)
/// - `per_page`: Items per page (default: 20, max: 100)
/// - `search`: Search query for school name or NPSN
/// - `status`: Filter by status (active, inactive)
/// 
/// # Returns
/// - `200 OK`: List of schools
/// - `401 Unauthorized`: Missing or invalid token
/// - `403 Forbidden`: User is not SuperAdmin
#[utoipa::path(
    get,
    path = "/api/v1/schools",
    tag = "Schools",
    params(
        ("page" = Option<i64>, Query, description = "Page number", example = 1),
        ("per_page" = Option<i64>, Query, description = "Items per page", example = 20),
        ("search" = Option<String>, Query, description = "Search query", example = "Jakarta"),
        ("status" = Option<String>, Query, description = "Filter by status", example = "active")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of schools", body = Vec<SchoolResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - SuperAdmin only", body = ErrorResponse)
    )
)]
pub async fn list_schools(
    auth_user: AuthUser,
    Query(params): Query<SearchParams>,
) -> AppResult<Json<Vec<SchoolResponse>>> {
    // Implementation
}
```

## Example 3: POST endpoint with path parameter

```rust
/// Create new registration
/// 
/// Creates a new student registration in draft status.
/// 
/// # Authentication
/// Requires JWT Bearer token with Parent role.
/// 
/// # Business Rules
/// - Registration starts in Draft status
/// - Period must be active
/// - Path quota must not be full
/// - Registration can be edited until submitted
/// 
/// # Returns
/// - `201 Created`: Registration created successfully
/// - `400 Bad Request`: Invalid input or business rule violation
/// - `401 Unauthorized`: Missing or invalid token
/// - `404 Not Found`: Period or path not found
#[utoipa::path(
    post,
    path = "/api/v1/registrations",
    tag = "Registrations",
    request_body = CreateRegistrationDto,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 201, description = "Registration created", body = RegistrationResponse),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Period not found", body = ErrorResponse)
    )
)]
pub async fn create_registration(
    auth_user: AuthUser,
    Json(payload): Json<CreateRegistrationDto>,
) -> AppResult<(StatusCode, Json<RegistrationResponse>)> {
    // Implementation
}
```

## Example 4: PUT endpoint with path parameter

```rust
/// Update school
/// 
/// Updates school information. Only SuperAdmin can update schools.
/// 
/// # Authentication
/// Requires JWT Bearer token with SuperAdmin role.
/// 
/// # Path Parameters
/// - `id`: School ID (UUID)
/// 
/// # Returns
/// - `200 OK`: School updated successfully
/// - `401 Unauthorized`: Missing or invalid token
/// - `403 Forbidden`: User is not SuperAdmin
/// - `404 Not Found`: School not found
#[utoipa::path(
    put,
    path = "/api/v1/schools/{id}",
    tag = "Schools",
    params(
        ("id" = i32, Path, description = "School ID", example = 1)
    ),
    request_body = UpdateSchoolDto,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "School updated", body = SchoolResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "School not found", body = ErrorResponse)
    )
)]
pub async fn update_school(
    auth_user: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateSchoolDto>,
) -> AppResult<Json<SchoolResponse>> {
    // Implementation
}
```

## Example 5: DELETE endpoint

```rust
/// Delete school
/// 
/// Soft deletes a school (sets status to inactive).
/// Only SuperAdmin can delete schools.
/// 
/// # Authentication
/// Requires JWT Bearer token with SuperAdmin role.
/// 
/// # Path Parameters
/// - `id`: School ID
/// 
/// # Returns
/// - `200 OK`: School deleted successfully
/// - `401 Unauthorized`: Missing or invalid token
/// - `403 Forbidden`: User is not SuperAdmin
/// - `404 Not Found`: School not found
#[utoipa::path(
    delete,
    path = "/api/v1/schools/{id}",
    tag = "Schools",
    params(
        ("id" = i32, Path, description = "School ID", example = 1)
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "School deleted", body = MessageResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "School not found", body = ErrorResponse)
    )
)]
pub async fn delete_school(
    auth_user: AuthUser,
    Path(id): Path<i32>,
) -> AppResult<Json<MessageResponse>> {
    // Implementation
}
```

## Example 6: Multipart file upload

```rust
/// Upload document
/// 
/// Uploads a document for a registration.
/// 
/// # Authentication
/// Requires JWT Bearer token with Parent role.
/// 
/// # Path Parameters
/// - `id`: Registration ID
/// 
/// # Request Body
/// Multipart form data with:
/// - `document_type`: Type of document (kartu_keluarga, akta_kelahiran, etc.)
/// - `file`: Document file (PDF or image, max 2MB)
/// 
/// # Returns
/// - `201 Created`: Document uploaded successfully
/// - `400 Bad Request`: Invalid file type or size
/// - `401 Unauthorized`: Missing or invalid token
/// - `403 Forbidden`: Not owner of registration
/// - `404 Not Found`: Registration not found
#[utoipa::path(
    post,
    path = "/api/v1/registrations/{id}/documents",
    tag = "Registrations",
    params(
        ("id" = i32, Path, description = "Registration ID", example = 1)
    ),
    request_body(
        content_type = "multipart/form-data",
        description = "Document file and metadata"
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 201, description = "Document uploaded", body = DocumentResponse),
        (status = 400, description = "Invalid file", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Registration not found", body = ErrorResponse)
    )
)]
pub async fn upload_document(
    auth_user: AuthUser,
    Path(id): Path<i32>,
    multipart: Multipart,
) -> AppResult<(StatusCode, Json<DocumentResponse>)> {
    // Implementation
}
```

## Example 7: Public endpoint (no authentication)

```rust
/// Check selection result
/// 
/// Public endpoint to check selection result using registration number and NISN.
/// 
/// # Authentication
/// No authentication required (public endpoint).
/// 
/// # Query Parameters
/// - `registration_number`: Registration number
/// - `nisn`: Student NISN
/// 
/// # Returns
/// - `200 OK`: Result found
/// - `404 Not Found`: Result not found or not announced yet
#[utoipa::path(
    get,
    path = "/api/v1/selection/result",
    tag = "Selection",
    params(
        ("registration_number" = String, Query, description = "Registration number", example = "REG-2024-001"),
        ("nisn" = String, Query, description = "Student NISN", example = "1234567890")
    ),
    responses(
        (status = 200, description = "Result found", body = CheckResultResponse),
        (status = 404, description = "Result not found", body = ErrorResponse)
    )
)]
pub async fn check_result(
    Query(params): Query<CheckResultRequest>,
) -> AppResult<Json<CheckResultResponse>> {
    // Implementation
}
```

## Tips for Good Documentation

### 1. Always add descriptions
```rust
/// Brief one-line description
/// 
/// Longer description with details about:
/// - What the endpoint does
/// - Business rules
/// - Special considerations
```

### 2. Document all response codes
```rust
responses(
    (status = 200, description = "Success", body = Response),
    (status = 400, description = "Bad request", body = ErrorResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Not found", body = ErrorResponse),
    (status = 500, description = "Internal error", body = ErrorResponse)
)
```

### 3. Add examples to schemas
```rust
#[derive(ToSchema)]
#[schema(example = json!({
    "field": "value"
}))]
pub struct MyDto {
    #[schema(example = "example value")]
    pub field: String,
}
```

### 4. Use proper HTTP methods
- `GET`: Retrieve data
- `POST`: Create new resource
- `PUT`: Update entire resource
- `PATCH`: Partial update
- `DELETE`: Delete resource

### 5. Group related endpoints with tags
```rust
#[utoipa::path(
    tag = "Authentication",  // Groups with other auth endpoints
    // ...
)]
```

### 6. Document security requirements
```rust
security(
    ("bearer_auth" = [])  // Requires JWT Bearer token
)
```

### 7. Add validation constraints
```rust
#[schema(
    min_length = 8,
    max_length = 100,
    pattern = "^[a-zA-Z0-9]+$",
    format = "email"
)]
```

## Common Patterns

### Pagination
```rust
params(
    ("page" = Option<i64>, Query, example = 1, minimum = 1),
    ("per_page" = Option<i64>, Query, example = 20, minimum = 1, maximum = 100)
)
```

### Search
```rust
params(
    ("search" = Option<String>, Query, example = "keyword")
)
```

### Filter by status
```rust
params(
    ("status" = Option<String>, Query, example = "active")
)
```

### Date range
```rust
params(
    ("start_date" = Option<String>, Query, example = "2024-01-01", format = "date"),
    ("end_date" = Option<String>, Query, example = "2024-12-31", format = "date")
)
```
