mod test_helpers;

use axum::http::StatusCode;
use serde_json::json;
use test_helpers::TestContext;

#[tokio::test]
async fn debug_school_create() {
    let ctx = TestContext::new().await;
    
    println!("=== Cleaning up test data ===");
    ctx.cleanup_test_data().await;
    
    println!("=== Creating Super Admin ===");
    let super_admin = ctx.create_super_admin().await;
    println!("Super Admin created: {}", super_admin.email);
    println!("Access Token: {}", &super_admin.access_token[..50]);
    
    // Test auth first
    println!("=== Testing auth with /auth/me ===");
    let (auth_status, auth_response) = ctx.get(
        "/api/v1/auth/me",
        Some(&super_admin.access_token)
    ).await;
    println!("Auth test status: {}", auth_status);
    println!("Auth test response: {:?}", auth_response);
    
    println!("=== Testing create school endpoint ===");
    let (status, response) = ctx.post(
        "/api/v1/schools",
        json!({
            "name": "Debug Test School",
            "npsn": "99999999",
            "code": "DEBUGSCH",
            "address": "Debug Test Address",
            "phone": "081234567890",
            "email": "debug_school@test.com"
        }),
        Some(&super_admin.access_token)
    ).await;
    
    println!("Status: {}", status);
    println!("Response: {:#?}", response);
    
    if status == StatusCode::CREATED {
        println!("✅ SUCCESS! School created");
        println!("School ID: {}", response["id"]);
        println!("School Code: {}", response["code"]);
        assert_eq!(response["name"], "Debug Test School");
        assert_eq!(response["status"], "active");
    } else {
        println!("❌ FAILED! Status: {}", status);
        println!("Error: {:?}", response["error"]);
        
        ctx.cleanup_test_data().await;
        panic!("School creation failed");
    }
    
    println!("=== Cleaning up ===");
    ctx.cleanup_test_data().await;
}
