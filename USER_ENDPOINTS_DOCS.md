# User Endpoints Documentation Template

Copy-paste these `#[utoipa::path]` macros above each endpoint function in `src/api/users.rs`:

## 1. create_user

```rust
/// Create new user
/// 
/// Creates a new user account. Only SchoolAdmin and SuperAdmin can create users.
/// SchoolAdmin can only create users for their own school.
/// 
/// # Authentication
/// Requires JWT Bearer token with SchoolAdmin or SuperAdmin role.
/// 
/// # Business Rules
/// - SuperAdmin can create users for any school
/// - SchoolAdmin can only create users for their own school
/// - Role must be one of: super_admin, school_admin, parent
/// - Email must be unique
/// 
/// # Returns
/// - `201 Created`: User created successfully
/// - `400 Bad Request`: Email already exists or invalid role
/// - `401 Unauthorized`: Missing or invalid token
/// - `403 Forbidden`: User is not admin
/// - `422 Unprocessable Entity`: Validation error
#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "Users",
    request_body = CreateUserRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 201, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Email already exists", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ValidationErrorResponse)
    )
)]
```

## 2. list_users

```rust
/// List users
/// 
/// Returns a paginated list of users. SuperAdmin sees all users,
/// SchoolAdmin only sees users from their school.
/// 
/// # Authentication
/// Requires JWT Bearer token.
/// 
/// # Query Parameters
/// - `page`: Page number (default: 1)
/// - `page_size`: Items per page (default: 10, max: 100)
/// - `search`: Search in name or email
/// - `role`: Filter by role
/// 
/// # Returns
/// - `200 OK`: List of users with pagination
/// - `401 Unauthorized`: Missing or invalid token
#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    params(ListUsersQuery),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of users", body = ListUsersResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
```

## 3. get_user

```rust
/// Get user details
/// 
/// Returns detailed information about a specific user.
/// 
/// # Authentication
/// Requires JWT Bearer token.
/// 
/// # Path Parameters
/// - `id`: User ID
/// 
/// # Returns
/// - `200 OK`: User details
/// - `401 Unauthorized`: Missing or invalid token
/// - `404 Not Found`: User not found
#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(
        ("id" = i32, Path, description = "User ID", example = 1)
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "User details", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    )
)]
```

## 4. get_current_user_full

```rust
/// Get current user profile
/// 
/// Returns complete profile information of the currently authenticated user.
/// 
/// # Authentication
/// Requires JWT Bearer token.
/// 
/// # Returns
/// - `200 OK`: Current user profile
/// - `401 Unauthorized`: Missing or invalid token
#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    tag = "Users",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Current user profile", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
```

## 5. update_user

```rust
/// Update user
/// 
/// Updates user information. Only name, phone, and NIK can be updated.
/// 
/// # Authentication
/// Requires JWT Bearer token.
/// 
/// # Path Parameters
/// - `id`: User ID
/// 
/// # Returns
/// - `200 OK`: User updated successfully
/// - `401 Unauthorized`: Missing or invalid token
/// - `404 Not Found`: User not found
/// - `422 Unprocessable Entity`: Validation error
#[utoipa::path(
    put,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(
        ("id" = i32, Path, description = "User ID", example = 1)
    ),
    request_body = UpdateUserRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "User updated successfully", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ValidationErrorResponse)
    )
)]
```

## 6. update_current_user

```rust
/// Update current user profile
/// 
/// Updates the profile of the currently authenticated user.
/// 
/// # Authentication
/// Requires JWT Bearer token.
/// 
/// # Returns
/// - `200 OK`: Profile updated successfully
/// - `401 Unauthorized`: Missing or invalid token
/// - `422 Unprocessable Entity`: Validation error
#[utoipa::path(
    put,
    path = "/api/v1/users/me",
    tag = "Users",
    request_body = UpdateUserRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Profile updated successfully", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ValidationErrorResponse)
    )
)]
```

## 7. delete_user

```rust
/// Delete user
/// 
/// Deletes a user account. Only admins can delete users.
/// 
/// # Authentication
/// Requires JWT Bearer token with SchoolAdmin or SuperAdmin role.
/// 
/// # Path Parameters
/// - `id`: User ID
/// 
/// # Returns
/// - `200 OK`: User deleted successfully
/// - `401 Unauthorized`: Missing or invalid token
/// - `403 Forbidden`: User is not admin
/// - `404 Not Found`: User not found
#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(
        ("id" = i32, Path, description = "User ID", example = 1)
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "User deleted successfully", body = MessageResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    )
)]
```

## 8. change_password

```rust
/// Change password
/// 
/// Changes the password of the currently authenticated user.
/// 
/// # Authentication
/// Requires JWT Bearer token.
/// 
/// # Returns
/// - `200 OK`: Password changed successfully
/// - `400 Bad Request`: Old password is incorrect
/// - `401 Unauthorized`: Missing or invalid token
/// - `422 Unprocessable Entity`: Validation error (password too short)
#[utoipa::path(
    post,
    path = "/api/v1/users/me/change-password",
    tag = "Users",
    request_body = ChangePasswordRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Password changed successfully", body = MessageResponse),
        (status = 400, description = "Old password is incorrect", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ValidationErrorResponse)
    )
)]
```

## Instructions

1. Add `pub` before each `async fn` to make them public
2. Copy the corresponding `#[utoipa::path]` macro above each function
3. Register all endpoints in `src/api/docs.rs` paths section
4. Register all DTOs in `src/api/docs.rs` components section
