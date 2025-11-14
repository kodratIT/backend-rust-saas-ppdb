mod common;

use axum::http::StatusCode;
use common::*;

async fn setup_test_period(state: &ppdb_backend::AppState, school_id: i32) -> i32 {
    // Create period
    let period_id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO periods (school_id, academic_year, level, start_date, end_date, status)
        VALUES ($1, '2024/2025', 'SMA', NOW(), NOW() + INTERVAL '3 months', 'active')
        RETURNING id
        "#,
    )
    .bind(school_id)
    .fetch_one(&state.db)
    .await
    .unwrap();

    // Create registration path
    sqlx::query(
        r#"
        INSERT INTO registration_paths (period_id, path_type, name, quota, scoring_config)
        VALUES ($1, 'zonasi', 'Jalur Zonasi', 100, '{"distance_weight": 2.0}')
        "#,
    )
    .bind(period_id)
    .execute(&state.db)
    .await
    .unwrap();

    period_id
}

#[tokio::test]
async fn test_create_registration_success() {
    let state = create_test_app_state().await;
    let mut router = create_test_router(state.clone());

    cleanup_test_data(&state.db).await;

    // Create school
    let school_id: i32 = sqlx::query_scalar(
        "INSERT INTO schools (name, npsn, code, status) VALUES ('Test School', '12345678', 'TESTSCH', 'active') RETURNING id"
    )
    .fetch_one(&state.db)
    .await
    .unwrap();

    // Setup period
    let period_id = setup_test_period(&state, school_id).await;
    
    // Get path_id
    let path_id: i32 = sqlx::query_scalar("SELECT id FROM registration_paths WHERE period_id = $1")
        .bind(period_id)
        .fetch_one(&state.db)
        .await
        .unwrap();

    // Register parent
    register_test_user(&mut router, "parent@test.com", "password123", "Parent User").await;
    
    // Update user to have school_id and verify email
    sqlx::query("UPDATE users SET school_id = $1, email_verified = true WHERE email = 'parent@test.com'")
        .bind(school_id)
        .execute(&state.db)
        .await
        .unwrap();

    let token = login_and_get_token(&mut router, "parent@test.com", "password123").await;

    // Create registration
    let body = serde_json::json!({
        "period_id": period_id,
        "path_id": path_id,
        "student_nisn": "1234567890",
        "student_name": "Ahmad Rizki",
        "student_gender": "L",
        "student_birth_place": "Jakarta",
        "student_birth_date": "2010-05-15T00:00:00Z",
        "student_religion": "Islam",
        "student_address": "Jl. Merdeka No. 10",
        "parent_name": "Parent User",
        "parent_nik": "3201234567890123",
        "parent_phone": "081234567890",
        "path_data": {
            "distance_km": 2.5
        }
    });

    let (status, response_body) = make_json_request(&mut router, "POST", "/registrations", Some(body), Some(&token)).await;

    assert_eq!(status, StatusCode::CREATED, "Response: {}", response_body);
    
    let response: serde_json::Value = parse_json_response(&response_body);
    assert_eq!(response["student_name"], "Ahmad Rizki");
    assert_eq!(response["status"], "draft");
}

#[tokio::test]
async fn test_registration_invalid_nisn() {
    let state = create_test_app_state().await;
    let mut router = create_test_router(state.clone());

    cleanup_test_data(&state.db).await;

    // Create school and period
    let school_id: i32 = sqlx::query_scalar(
        "INSERT INTO schools (name, npsn, code, status) VALUES ('Test School', '12345678', 'TESTSCH', 'active') RETURNING id"
    )
    .fetch_one(&state.db)
    .await
    .unwrap();

    let period_id = setup_test_period(&state, school_id).await;
    let path_id: i32 = sqlx::query_scalar("SELECT id FROM registration_paths WHERE period_id = $1")
        .bind(period_id)
        .fetch_one(&state.db)
        .await
        .unwrap();

    // Register parent
    register_test_user(&mut router, "parent2@test.com", "password123", "Parent User").await;
    sqlx::query("UPDATE users SET school_id = $1, email_verified = true WHERE email = 'parent2@test.com'")
        .bind(school_id)
        .execute(&state.db)
        .await
        .unwrap();

    let token = login_and_get_token(&mut router, "parent2@test.com", "password123").await;

    // Try to create registration with invalid NISN (not 10 digits)
    let body = serde_json::json!({
        "period_id": period_id,
        "path_id": path_id,
        "student_nisn": "123",  // Invalid: not 10 digits
        "student_name": "Ahmad Rizki",
        "student_gender": "L",
        "student_birth_place": "Jakarta",
        "student_birth_date": "2010-05-15T00:00:00Z",
        "student_religion": "Islam",
        "student_address": "Jl. Merdeka No. 10",
        "parent_name": "Parent User",
        "parent_nik": "3201234567890123",
        "parent_phone": "081234567890",
        "path_data": {
            "distance_km": 2.5
        }
    });

    let (status, body) = make_json_request(&mut router, "POST", "/registrations", Some(body), Some(&token)).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("Validation error") || body.contains("NISN"));
}

#[tokio::test]
async fn test_submit_registration() {
    let state = create_test_app_state().await;
    let mut router = create_test_router(state.clone());

    cleanup_test_data(&state.db).await;

    // Setup
    let school_id: i32 = sqlx::query_scalar(
        "INSERT INTO schools (name, npsn, code, status) VALUES ('Test School', '12345678', 'TESTSCH', 'active') RETURNING id"
    )
    .fetch_one(&state.db)
    .await
    .unwrap();

    let period_id = setup_test_period(&state, school_id).await;
    let path_id: i32 = sqlx::query_scalar("SELECT id FROM registration_paths WHERE period_id = $1")
        .bind(period_id)
        .fetch_one(&state.db)
        .await
        .unwrap();

    register_test_user(&mut router, "parent3@test.com", "password123", "Parent User").await;
    sqlx::query("UPDATE users SET school_id = $1, email_verified = true WHERE email = 'parent3@test.com'")
        .bind(school_id)
        .execute(&state.db)
        .await
        .unwrap();

    let token = login_and_get_token(&mut router, "parent3@test.com", "password123").await;

    // Create registration
    let body = serde_json::json!({
        "period_id": period_id,
        "path_id": path_id,
        "student_nisn": "9876543210",
        "student_name": "Siti Nurhaliza",
        "student_gender": "P",
        "student_birth_place": "Bandung",
        "student_birth_date": "2010-08-20T00:00:00Z",
        "student_religion": "Islam",
        "student_address": "Jl. Asia Afrika No. 5",
        "parent_name": "Parent User",
        "parent_nik": "3201234567890123",
        "parent_phone": "081234567890",
        "path_data": {
            "distance_km": 1.5
        }
    });

    let (_, response_body) = make_json_request(&mut router, "POST", "/registrations", Some(body), Some(&token)).await;
    let response: serde_json::Value = parse_json_response(&response_body);
    let registration_id = response["id"].as_i64().unwrap();

    // Upload a document first
    let user_id: i32 = sqlx::query_scalar("SELECT id FROM users WHERE email = 'parent3@test.com'")
        .fetch_one(&state.db)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO documents (registration_id, document_type, file_url, file_name, file_size, mime_type, verification_status) 
         VALUES ($1, 'kartu_keluarga', 'http://example.com/kk.pdf', 'kk.pdf', 1000000, 'application/pdf', 'pending')"
    )
    .bind(registration_id)
    .execute(&state.db)
    .await
    .unwrap();

    // Submit registration
    let (status, response_body) = make_json_request::<()>(
        &mut router,
        "POST",
        &format!("/registrations/{}/submit", registration_id),
        None,
        Some(&token),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "Response: {}", response_body);
    
    let response: serde_json::Value = parse_json_response(&response_body);
    assert_eq!(response["status"], "submitted");
    assert!(response["registration_number"].is_string());
}

#[tokio::test]
async fn test_list_registrations_parent_only_sees_own() {
    let state = create_test_app_state().await;
    let mut router = create_test_router(state.clone());

    cleanup_test_data(&state.db).await;

    // Setup
    let school_id: i32 = sqlx::query_scalar(
        "INSERT INTO schools (name, npsn, code, status) VALUES ('Test School', '12345678', 'TESTSCH', 'active') RETURNING id"
    )
    .fetch_one(&state.db)
    .await
    .unwrap();

    let period_id = setup_test_period(&state, school_id).await;
    let path_id: i32 = sqlx::query_scalar("SELECT id FROM registration_paths WHERE period_id = $1")
        .bind(period_id)
        .fetch_one(&state.db)
        .await
        .unwrap();

    // Create two parents
    register_test_user(&mut router, "parent4@test.com", "password123", "Parent 4").await;
    register_test_user(&mut router, "parent5@test.com", "password123", "Parent 5").await;
    
    sqlx::query("UPDATE users SET school_id = $1, email_verified = true WHERE email IN ('parent4@test.com', 'parent5@test.com')")
        .bind(school_id)
        .execute(&state.db)
        .await
        .unwrap();

    let token1 = login_and_get_token(&mut router, "parent4@test.com", "password123").await;
    let token2 = login_and_get_token(&mut router, "parent5@test.com", "password123").await;

    // Parent 1 creates registration
    let body = serde_json::json!({
        "period_id": period_id,
        "path_id": path_id,
        "student_nisn": "1111111111",
        "student_name": "Student 1",
        "student_gender": "L",
        "student_birth_place": "Jakarta",
        "student_birth_date": "2010-01-01T00:00:00Z",
        "student_religion": "Islam",
        "student_address": "Address 1",
        "parent_name": "Parent 4",
        "parent_nik": "3201234567890123",
        "parent_phone": "081234567890",
        "path_data": {"distance_km": 1.0}
    });

    make_json_request(&mut router, "POST", "/registrations", Some(body), Some(&token1)).await;

    // Parent 2 lists registrations - should not see Parent 1's registration
    let (status, response_body) = make_json_request::<()>(
        &mut router,
        "GET",
        "/registrations",
        None,
        Some(&token2),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = parse_json_response(&response_body);
    assert_eq!(response["registrations"].as_array().unwrap().len(), 0);
}
