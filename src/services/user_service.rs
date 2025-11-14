use crate::models::user::User;
use crate::repositories::user_repo::UserRepository;
use crate::utils::error::{AppError, AppResult};
use crate::utils::password;

pub struct UserService {
    user_repo: UserRepository,
}

impl UserService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }

    pub async fn create_user(
        &self,
        school_id: Option<i32>,
        email: String,
        password: String,
        full_name: String,
        phone: Option<String>,
        nik: Option<String>,
        role: String,
    ) -> AppResult<User> {
        // Check if email already exists
        if let Some(_) = self.user_repo.find_by_email(&email).await? {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        // Hash password
        let password_hash = password::hash_password(&password)?;

        // Create user
        let user = self
            .user_repo
            .create_user(
                school_id,
                &email,
                &password_hash,
                &full_name,
                phone.as_deref(),
                nik.as_deref(),
                &role,
            )
            .await?;

        Ok(user)
    }

    pub async fn list_users(
        &self,
        school_id: Option<i32>,
        page: i64,
        page_size: i64,
        search: Option<String>,
        role: Option<String>,
    ) -> AppResult<(Vec<User>, i64)> {
        let offset = (page - 1) * page_size;

        let users = self
            .user_repo
            .find_all(school_id, page_size, offset, search.clone(), role.clone())
            .await?;

        let total = self
            .user_repo
            .count_all(school_id, search, role)
            .await?;

        Ok((users, total))
    }

    pub async fn get_user(&self, id: i32) -> AppResult<User> {
        self.user_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    pub async fn update_user(
        &self,
        id: i32,
        full_name: Option<String>,
        phone: Option<String>,
        nik: Option<String>,
    ) -> AppResult<User> {
        // Check if user exists
        let _user = self.get_user(id).await?;

        // Update user
        let updated_user = self
            .user_repo
            .update_user(id, full_name.as_deref(), phone.as_deref(), nik.as_deref())
            .await?;

        Ok(updated_user)
    }

    pub async fn delete_user(&self, id: i32) -> AppResult<()> {
        // Check if user exists
        let _user = self.get_user(id).await?;

        // Delete user
        self.user_repo.delete_user(id).await?;

        Ok(())
    }

    pub async fn change_password(
        &self,
        id: i32,
        old_password: String,
        new_password: String,
    ) -> AppResult<()> {
        // Get user
        let user = self.get_user(id).await?;

        // Verify old password
        let is_valid = password::verify_password(&old_password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::Authentication("Invalid old password".to_string()));
        }

        // Hash new password
        let password_hash = password::hash_password(&new_password)?;

        // Update password
        self.user_repo.update_password(id, &password_hash).await?;

        Ok(())
    }
}
