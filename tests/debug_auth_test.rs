mod test_helpers;

use axum::http::StatusCode;
use serde_json::json;
use test_helpers::TestContext;

#[tokio::test]
async fn debug_auth_register() {
    let ctx = TestContext::new().await;
    
    println!("=== Cleaning up test data ===");
    ctx.cleanup_test_data().await;
    
    println!("=== Testing register endpoint ===");
    let (status, response) = ctx.post(
        "/api/v1/auth/register",
        json!({
            "email": "debug_test@test.com",
            "password": "password123",
            "full_name": "Debug Test User",
            "phone": "081234567890",
            "nik": "1234567890123456"
        }),
        None
    ).await;
    
    println!("Status: {}", status);
    println!("Response: {:#?}", response);
    
    if status == StatusCode::CREATED {
        println!("✅ SUCCESS! User created");
        assert_eq!(response["email"], "debug_test@test.com");
    } else {
        println!("❌ FAILED! Status: {}", status);
        println!("Error: {:?}", response["error"]);
        
        // Cleanup even on failure
        ctx.cleanup_test_data().await;
        
        panic!("Registration failed");
    }
    
    println!("=== Cleaning up ===");
    ctx.cleanup_test_data().await;
}
