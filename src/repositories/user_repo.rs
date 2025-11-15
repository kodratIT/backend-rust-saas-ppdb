use sqlx::PgPool;

use crate::models::user::User;
use crate::utils::error::{AppError, AppResult};

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(
        &self,
        school_id: Option<i32>,
        email: &str,
        password_hash: &str,
        full_name: &str,
        phone: Option<&str>,
        nik: Option<&str>,
        role: &str,
    ) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (school_id, email, password_hash, full_name, phone, nik, role)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(school_id)
        .bind(email)
        .bind(password_hash)
        .bind(full_name)
        .bind(phone)
        .bind(nik)
        .bind(role)
        .persistent(false)  // Disable prepared statement
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                AppError::Conflict("Email already exists".to_string())
            }
            _ => AppError::Database(e),
        })?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE email = $1
            "#,
        )
        .bind(email)
        .persistent(false)  // Disable prepared statement
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: i32) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE id = $1
            "#,
        )
        .bind(id)
        .persistent(false)  // Disable prepared statement
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_user(
        &self,
        id: i32,
        full_name: Option<&str>,
        phone: Option<&str>,
        nik: Option<&str>,
    ) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET full_name = COALESCE($2, full_name),
                phone = COALESCE($3, phone),
                nik = COALESCE($4, nik),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(full_name)
        .bind(phone)
        .bind(nik)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_all(
        &self,
        school_id: Option<i32>,
        page_size: i64,
        offset: i64,
        search: Option<String>,
        role: Option<String>,
    ) -> AppResult<Vec<User>> {
        let mut conditions = vec!["1=1".to_string()];
        let mut param_index = 3; // Start from 3 because 1 and 2 are for LIMIT and OFFSET
        
        if school_id.is_some() {
            conditions.push(format!("school_id = ${}", param_index));
            param_index += 1;
        }
        
        if search.is_some() {
            conditions.push(format!("(full_name ILIKE '%' || ${} || '%' OR email ILIKE '%' || ${} || '%')", param_index, param_index));
            param_index += 1;
        }
        
        if role.is_some() {
            conditions.push(format!("role = ${}", param_index));
        }
        
        let query = format!(
            "SELECT * FROM users WHERE {} ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            conditions.join(" AND ")
        );

        let mut q = sqlx::query_as::<_, User>(&query)
            .bind(page_size)
            .bind(offset);
            
        if let Some(sid) = school_id {
            q = q.bind(sid);
        }
        
        if let Some(s) = search {
            q = q.bind(s);
        }
        
        if let Some(r) = role {
            q = q.bind(r);
        }

        let users = q.fetch_all(&self.pool).await?;

        Ok(users)
    }

    pub async fn count_all(
        &self,
        school_id: Option<i32>,
        search: Option<String>,
        role: Option<String>,
    ) -> AppResult<i64> {
        let mut conditions = vec!["1=1".to_string()];
        let mut param_index = 1;
        
        if school_id.is_some() {
            conditions.push(format!("school_id = ${}", param_index));
            param_index += 1;
        }
        
        if search.is_some() {
            conditions.push(format!("(full_name ILIKE '%' || ${} || '%' OR email ILIKE '%' || ${} || '%')", param_index, param_index));
            param_index += 1;
        }
        
        if role.is_some() {
            conditions.push(format!("role = ${}", param_index));
        }

        let query = format!(
            "SELECT COUNT(*) FROM users WHERE {}",
            conditions.join(" AND ")
        );

        let mut q = sqlx::query_scalar::<_, i64>(&query);
            
        if let Some(sid) = school_id {
            q = q.bind(sid);
        }
        
        if let Some(s) = search {
            q = q.bind(s);
        }
        
        if let Some(r) = role {
            q = q.bind(r);
        }

        let count = q.fetch_one(&self.pool).await?;

        Ok(count)
    }

    pub async fn delete_user(&self, id: i32) -> AppResult<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn set_email_verified(&self, id: i32) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET email_verified = true,
                email_verification_token = NULL,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn set_email_verification_token(&self, id: i32, token: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET email_verification_token = $2,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(token)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_verification_token(&self, token: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE email_verification_token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn set_reset_password_token(
        &self,
        email: &str,
        token: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET reset_password_token = $2,
                reset_password_expires = $3,
                updated_at = CURRENT_TIMESTAMP
            WHERE email = $1
            "#,
        )
        .bind(email)
        .bind(token)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_reset_token(&self, token: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users 
            WHERE reset_password_token = $1 
            AND reset_password_expires > CURRENT_TIMESTAMP
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_password(&self, id: i32, password_hash: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $2,
                reset_password_token = NULL,
                reset_password_expires = NULL,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(password_hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_last_login(&self, id: i32) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET last_login_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
