use sqlx::PgPool;

use crate::models::school::School;
use crate::utils::error::{AppError, AppResult};

pub struct SchoolRepository {
    pool: PgPool,
}

impl SchoolRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i32) -> AppResult<Option<School>> {
        let school = sqlx::query_as::<_, School>(
            r#"
            SELECT * FROM schools WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(school)
    }

    pub async fn find_all(
        &self,
        page_size: i64,
        offset: i64,
        search: Option<String>,
        status: Option<String>,
    ) -> AppResult<Vec<School>> {
        let mut query = String::from("SELECT * FROM schools WHERE 1=1");
        
        if search.is_some() {
            query.push_str(" AND (name ILIKE '%' || $3 || '%' OR npsn ILIKE '%' || $3 || '%')");
        }
        
        if status.is_some() {
            query.push_str(" AND status = $4");
        }
        
        query.push_str(" ORDER BY created_at DESC LIMIT $1 OFFSET $2");

        let mut q = sqlx::query_as::<_, School>(&query)
            .bind(page_size)
            .bind(offset);
            
        if let Some(s) = search {
            q = q.bind(s);
        }
        
        if let Some(st) = status {
            q = q.bind(st);
        }

        let schools = q.fetch_all(&self.pool).await?;

        Ok(schools)
    }

    pub async fn count_all(
        &self,
        search: Option<String>,
        status: Option<String>,
    ) -> AppResult<i64> {
        let mut query = String::from("SELECT COUNT(*) FROM schools WHERE 1=1");
        
        if search.is_some() {
            query.push_str(" AND (name ILIKE '%' || $1 || '%' OR npsn ILIKE '%' || $1 || '%')");
        }
        
        if status.is_some() {
            query.push_str(" AND status = $2");
        }

        let mut q = sqlx::query_scalar::<_, i64>(&query);
            
        if let Some(s) = search {
            q = q.bind(s);
        }
        
        if let Some(st) = status {
            q = q.bind(st);
        }

        let count = q.fetch_one(&self.pool).await?;

        Ok(count)
    }

    pub async fn create_school(
        &self,
        name: &str,
        npsn: &str,
        code: &str,
        address: &str,
        phone: Option<&str>,
        email: Option<&str>,
    ) -> AppResult<School> {
        let school = sqlx::query_as::<_, School>(
            r#"
            INSERT INTO schools (name, npsn, code, address, phone, email)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(npsn)
        .bind(code)
        .bind(address)
        .bind(phone)
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                AppError::Conflict("School with this NPSN or code already exists".to_string())
            }
            _ => AppError::Database(e),
        })?;

        Ok(school)
    }

    pub async fn update_school(
        &self,
        id: i32,
        name: Option<&str>,
        address: Option<&str>,
        phone: Option<&str>,
        email: Option<&str>,
    ) -> AppResult<School> {
        let school = sqlx::query_as::<_, School>(
            r#"
            UPDATE schools
            SET name = COALESCE($2, name),
                address = COALESCE($3, address),
                phone = COALESCE($4, phone),
                email = COALESCE($5, email),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(address)
        .bind(phone)
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        Ok(school)
    }

    pub async fn delete_school(&self, id: i32) -> AppResult<()> {
        sqlx::query("DELETE FROM schools WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn find_by_npsn(&self, npsn: &str) -> AppResult<Option<School>> {
        let school = sqlx::query_as::<_, School>(
            r#"
            SELECT * FROM schools WHERE npsn = $1
            "#,
        )
        .bind(npsn)
        .fetch_optional(&self.pool)
        .await?;

        Ok(school)
    }

    pub async fn find_by_code(&self, code: &str) -> AppResult<Option<School>> {
        let school = sqlx::query_as::<_, School>(
            r#"
            SELECT * FROM schools WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;

        Ok(school)
    }

    pub async fn update_status(&self, id: i32, status: &str) -> AppResult<School> {
        let school = sqlx::query_as::<_, School>(
            r#"
            UPDATE schools
            SET status = $2,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        Ok(school)
    }
}
