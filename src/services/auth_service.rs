use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::config::Config;
use crate::dto::auth_dto::{AuthResponse, LoginRequest, RegisterRequest, UserResponse};
use crate::repositories::user_repo::UserRepository;
use crate::utils::error::{AppError, AppResult};
use crate::utils::{jwt, password};

pub struct AuthService {
    user_repo: UserRepository,
    config: Config,
}

impl AuthService {
    pub fn new(user_repo: UserRepository, config: Config) -> Self {
        Self { user_repo, config }
    }

    pub async fn register(&self, req: RegisterRequest) -> AppResult<UserResponse> {
        // Check if email already exists
        if let Some(_) = self.user_repo.find_by_email(&req.email).await? {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        // Hash password
        let password_hash = password::hash_password(&req.password)?;

        // Generate verification token
        let verification_token = Uuid::new_v4().to_string();

        // Create user (default role: parent, no school_id)
        let user = self
            .user_repo
            .create_user(
                None,
                &req.email,
                &password_hash,
                &req.full_name,
                req.phone.as_deref(),
                req.nik.as_deref(),
                "parent",
            )
            .await?;

        // Set verification token
        self.user_repo
            .set_email_verification_token(user.id, &verification_token)
            .await?;

        // TODO: Send verification email
        tracing::info!(
            "Verification token for {}: {}",
            user.email,
            verification_token
        );

        Ok(UserResponse {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            role: user.role,
            school_id: user.school_id,
        })
    }

    pub async fn login(&self, req: LoginRequest) -> AppResult<AuthResponse> {
        // Find user by email
        let user = self
            .user_repo
            .find_by_email(&req.email)
            .await?
            .ok_or_else(|| AppError::Authentication("Invalid credentials".to_string()))?;

        // Verify password
        let is_valid = password::verify_password(&req.password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::Authentication("Invalid credentials".to_string()));
        }

        // Check if email is verified
        if !user.email_verified {
            return Err(AppError::Authentication(
                "Email not verified. Please check your email.".to_string(),
            ));
        }

        // Update last login
        self.user_repo.update_last_login(user.id).await?;

        // Generate access token
        let access_token = jwt::generate_token(
            user.id,
            user.email.clone(),
            user.role.clone(),
            user.school_id,
            &self.config.jwt_secret,
            self.config.jwt_expiration_hours,
        )?;

        // Generate refresh token
        let refresh_token = jwt::generate_refresh_token(
            user.id,
            user.email.clone(),
            user.role.clone(),
            user.school_id,
            &self.config.jwt_secret,
        )?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.jwt_expiration_hours * 3600, // Convert to seconds
            user: UserResponse {
                id: user.id,
                email: user.email,
                full_name: user.full_name,
                role: user.role,
                school_id: user.school_id,
            },
        })
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> AppResult<crate::dto::auth_dto::RefreshTokenResponse> {
        // Verify refresh token
        let claims = jwt::verify_refresh_token(refresh_token, &self.config.jwt_secret)?;

        // Find user to ensure they still exist and are active
        let user = self
            .user_repo
            .find_by_id(claims.sub)
            .await?
            .ok_or_else(|| AppError::Authentication("User not found".to_string()))?;

        // Check if email is still verified
        if !user.email_verified {
            return Err(AppError::Authentication("User account is not active".to_string()));
        }

        // Generate new access token
        let access_token = jwt::generate_token(
            user.id,
            user.email,
            user.role,
            user.school_id,
            &self.config.jwt_secret,
            self.config.jwt_expiration_hours,
        )?;

        Ok(crate::dto::auth_dto::RefreshTokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.jwt_expiration_hours * 3600,
        })
    }

    pub async fn verify_email(&self, token: &str) -> AppResult<UserResponse> {
        // Find user by verification token
        let user = self
            .user_repo
            .find_by_verification_token(token)
            .await?
            .ok_or_else(|| AppError::NotFound("Invalid verification token".to_string()))?;

        // Check if already verified
        if user.email_verified {
            return Err(AppError::Validation(
                "Email already verified".to_string(),
            ));
        }

        // Set email as verified
        let updated_user = self.user_repo.set_email_verified(user.id).await?;

        Ok(UserResponse {
            id: updated_user.id,
            email: updated_user.email,
            full_name: updated_user.full_name,
            role: updated_user.role,
            school_id: updated_user.school_id,
        })
    }

    pub async fn forgot_password(&self, email: &str) -> AppResult<()> {
        // Find user by email
        let user = self
            .user_repo
            .find_by_email(email)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Generate reset token
        let reset_token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::hours(1);

        // Set reset token
        self.user_repo
            .set_reset_password_token(email, &reset_token, expires_at)
            .await?;

        // TODO: Send reset password email
        tracing::info!("Reset token for {}: {}", email, reset_token);

        Ok(())
    }

    pub async fn reset_password(&self, token: &str, new_password: &str) -> AppResult<()> {
        // Find user by reset token
        let user = self
            .user_repo
            .find_by_reset_token(token)
            .await?
            .ok_or_else(|| {
                AppError::NotFound("Invalid or expired reset token".to_string())
            })?;

        // Hash new password
        let password_hash = password::hash_password(new_password)?;

        // Update password
        self.user_repo.update_password(user.id, &password_hash).await?;

        Ok(())
    }

    pub async fn get_user_by_id(&self, id: i32) -> AppResult<UserResponse> {
        let user = self
            .user_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(UserResponse {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            role: user.role,
            school_id: user.school_id,
        })
    }
}
