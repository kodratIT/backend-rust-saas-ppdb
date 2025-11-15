use axum::Router;
use axum::body::Body;
use axum::http::{Request, Method, StatusCode};
use once_cell::sync::Lazy;
use ppdb_backend::{api, utils::password::hash_password, AppState, Config};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;

// Shared test database pool
static TEST_DB: Lazy<Arc<Mutex<Option<sqlx::PgPool>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

#[derive(Clone)]
pub struct TestContext {
    pub app: Router,
    pub db: sqlx::PgPool,
}

#[derive(Debug, Deserialize)]
pub struct TestAuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: TestUserResponse,
}

#[derive(Debug, Deserialize)]
pub struct TestUserResponse {
    pub id: i32,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub school_id: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct TestRegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub nik: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TestLoginRequest {
    pub email: String,
    pub password: String,
}

pub struct TestUser {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub access_token: String,
    pub refresh_token: String,
    pub role: String,
    pub school_id: Option<i32>,
}

impl TestContext {
    pub async fn new() -> Self {
        // Load test environment
        dotenvy::dotenv().ok();
        
        let config = Config::from_env().expect("Failed to load config");
        
        // Get or create database pool
        let mut db_guard = TEST_DB.lock().await;
        let db = if let Some(pool) = db_guard.as_ref() {
            pool.clone()
        } else {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&config.database_url)
                .await
                .expect("Failed to connect to test database");
            
            *db_guard = Some(pool.clone());
            pool
        };
        drop(db_guard);

        // Create test app
        let app_state = AppState {
            db: db.clone(),
            config: config.clone(),
        };

        let app = Router::new().nest("/api/v1", api::routes(app_state));

        TestContext { app, db }
    }

    pub async fn request(&self, method: Method, uri: &str, body: Option<Value>, auth_token: Option<&str>) -> (StatusCode, Value) {
        let mut request_builder = Request::builder()
            .method(method)
            .uri(uri);

        if let Some(token) = auth_token {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
        }

        let request = if let Some(json_body) = body {
            request_builder
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&json_body).unwrap()))
                .unwrap()
        } else {
            request_builder.body(Body::empty()).unwrap()
        };

        let response = self.app.clone().oneshot(request).await.unwrap();
        
        let status = response.status();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        
        let body_value: Value = if body_bytes.is_empty() {
            json!({})
        } else {
            serde_json::from_slice(&body_bytes).unwrap_or_else(|e| {
                let body_str = String::from_utf8_lossy(&body_bytes).to_string();
                eprintln!("Failed to parse JSON: {}. Body: {}", e, body_str);
                json!({ "error": body_str })
            })
        };

        if !status.is_success() {
            eprintln!("Request failed with status {}: {:?}", status, body_value);
        }

        (status, body_value)
    }

    pub async fn post(&self, uri: &str, body: Value, auth_token: Option<&str>) -> (StatusCode, Value) {
        self.request(Method::POST, uri, Some(body), auth_token).await
    }

    pub async fn get(&self, uri: &str, auth_token: Option<&str>) -> (StatusCode, Value) {
        self.request(Method::GET, uri, None, auth_token).await
    }

    pub async fn put(&self, uri: &str, body: Value, auth_token: Option<&str>) -> (StatusCode, Value) {
        self.request(Method::PUT, uri, Some(body), auth_token).await
    }

    pub async fn delete(&self, uri: &str, auth_token: Option<&str>) -> (StatusCode, Value) {
        self.request(Method::DELETE, uri, None, auth_token).await
    }



    pub async fn create_super_admin(&self) -> TestUser {
        // Create super admin directly in database
        let password_hash = hash_password("password123").expect("Failed to hash password");

        let user = sqlx::query!(
            r#"
            INSERT INTO users (email, password_hash, full_name, role, email_verified)
            VALUES ($1, $2, $3, 'super_admin', true)
            ON CONFLICT (email) DO UPDATE 
            SET password_hash = EXCLUDED.password_hash,
                email_verified = true
            RETURNING id, email, full_name, role, school_id
            "#,
            "superadmin@test.com",
            password_hash,
            "Super Admin"
        )
        .fetch_one(&self.db)
        .await
        .expect("Failed to create super admin");

        // Login to get tokens
        let (status, response) = self.post(
            "/api/v1/auth/login",
            json!({
                "email": "superadmin@test.com",
                "password": "password123"
            }),
            None
        ).await;

        assert_eq!(status, StatusCode::OK, "Login failed: {:?}", response);
        
        let login_response: TestAuthResponse = serde_json::from_value(response)
            .expect("Failed to parse login response");

        TestUser {
            id: user.id,
            email: user.email,
            password: "password123".to_string(),
            access_token: login_response.access_token,
            refresh_token: login_response.refresh_token,
            role: user.role,
            school_id: user.school_id,
        }
    }

    pub async fn create_test_school(&self, token: &str, code: &str) -> i32 {
        // Generate 8-digit NPSN from code hash
        let npsn = format!("{:08}", code.chars().map(|c| c as u32).sum::<u32>() % 100000000);
        
        let (status, response) = self.post(
            "/api/v1/schools",
            json!({
                "name": format!("Test School {}", code),
                "npsn": npsn,
                "code": code,
                "address": "Test Address",
                "phone": "081234567890",
                "email": format!("school{}@test.com", code)
            }),
            Some(token)
        ).await;

        if status != StatusCode::CREATED {
            panic!("Failed to create test school. Status: {}, Response: {:?}", status, response);
        }

        response["id"].as_i64().expect("School ID not found in response") as i32
    }

    pub async fn create_school_admin(&self, school_id: i32, email: &str) -> TestUser {
        // Create school admin directly in database
        let password_hash = hash_password("password123").expect("Failed to hash password");

        let user = sqlx::query!(
            r#"
            INSERT INTO users (email, password_hash, full_name, role, school_id, email_verified)
            VALUES ($1, $2, $3, 'school_admin', $4, true)
            ON CONFLICT (email) DO UPDATE 
            SET password_hash = EXCLUDED.password_hash,
                email_verified = true,
                school_id = EXCLUDED.school_id
            RETURNING id, email, full_name, role, school_id
            "#,
            email,
            password_hash,
            "School Admin",
            school_id
        )
        .fetch_one(&self.db)
        .await
        .expect("Failed to create school admin");

        // Login to get tokens
        let (status, response) = self.post(
            "/api/v1/auth/login",
            json!({
                "email": email,
                "password": "password123"
            }),
            None
        ).await;

        assert_eq!(status, StatusCode::OK, "Login failed: {:?}", response);
        
        let login_response: TestAuthResponse = serde_json::from_value(response)
            .expect("Failed to parse login response");

        TestUser {
            id: user.id,
            email: user.email,
            password: "password123".to_string(),
            access_token: login_response.access_token,
            refresh_token: login_response.refresh_token,
            role: user.role,
            school_id: user.school_id,
        }
    }

    pub async fn create_parent(&self, school_id: i32, email: &str) -> TestUser {
        let password_hash = hash_password("password123").expect("Failed to hash password");

        let user = sqlx::query!(
            r#"
            INSERT INTO users (email, password_hash, full_name, role, school_id, email_verified, phone, nik)
            VALUES ($1, $2, $3, 'parent', $4, true, $5, $6)
            ON CONFLICT (email) DO UPDATE 
            SET password_hash = EXCLUDED.password_hash,
                email_verified = true,
                school_id = EXCLUDED.school_id
            RETURNING id, email, full_name, role, school_id
            "#,
            email,
            password_hash,
            "Test Parent",
            school_id,
            "081234567890",
            "1234567890123456"
        )
        .fetch_one(&self.db)
        .await
        .expect("Failed to create parent");

        let (status, response) = self.post(
            "/api/v1/auth/login",
            json!({
                "email": email,
                "password": "password123"
            }),
            None
        ).await;

        assert_eq!(status, StatusCode::OK, "Login failed: {:?}", response);
        
        let login_response: TestAuthResponse = serde_json::from_value(response)
            .expect("Failed to parse login response");

        TestUser {
            id: user.id,
            email: user.email,
            password: "password123".to_string(),
            access_token: login_response.access_token,
            refresh_token: login_response.refresh_token,
            role: user.role,
            school_id: user.school_id,
        }
    }

    pub async fn cleanup_test_data(&self) {
        // Clean up test data in correct order to respect foreign keys
        // Use raw queries to avoid compile-time checking issues
        
        // Delete documents for test registrations
        sqlx::query(
            "DELETE FROM documents WHERE registration_id IN 
             (SELECT id FROM registrations WHERE student_name LIKE '%Test%' OR student_name LIKE '%Student%')"
        )
        .execute(&self.db)
        .await
        .ok();

        // Delete test registrations
        sqlx::query(
            "DELETE FROM registrations WHERE student_name LIKE '%Test%' 
             OR student_name LIKE '%Student%'
             OR student_nisn LIKE '1%'
             OR student_nisn LIKE '2%'"
        )
        .execute(&self.db)
        .await
        .ok();

        // Delete registration paths from test periods
        sqlx::query(
            "DELETE FROM registration_paths WHERE period_id IN 
             (SELECT id FROM periods WHERE school_id IN 
              (SELECT id FROM schools WHERE code LIKE 'TEST%'))"
        )
        .execute(&self.db)
        .await
        .ok();

        // Delete test periods
        sqlx::query(
            "DELETE FROM periods WHERE school_id IN 
             (SELECT id FROM schools WHERE code LIKE 'TEST%')"
        )
        .execute(&self.db)
        .await
        .ok();

        // Delete test users
        sqlx::query("DELETE FROM users WHERE email LIKE '%@test.com'")
            .execute(&self.db)
            .await
            .ok();

        // Delete test schools
        sqlx::query("DELETE FROM schools WHERE code LIKE 'TEST%' OR npsn LIKE '1234567%'")
            .execute(&self.db)
            .await
            .ok();
    }
}

pub fn parse_json<T: DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value.clone()).unwrap_or_else(|e| {
        panic!(
            "Failed to parse JSON: {}. Value: {}",
            e, value
        );
    })
}
