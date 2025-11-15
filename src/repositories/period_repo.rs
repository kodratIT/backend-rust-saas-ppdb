use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;

use crate::models::period::{Period, RegistrationPath};
use crate::utils::error::AppResult;

pub struct PeriodRepository {
    pool: PgPool,
}

impl PeriodRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_period(
        &self,
        school_id: i32,
        academic_year: &str,
        level: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        reenrollment_deadline: Option<NaiveDate>,
    ) -> AppResult<Period> {
        // Use start_date and end_date for registration dates as well
        let period = sqlx::query_as::<_, Period>(
            r#"
            INSERT INTO periods (school_id, academic_year, level, start_date, end_date, registration_start, registration_end, reenrollment_deadline, status)
            VALUES ($1, $2, $3, $4, $5, $4, $5, $6, 'draft')
            RETURNING *
            "#,
        )
        .bind(school_id)
        .bind(academic_year)
        .bind(level)
        .bind(start_date)
        .bind(end_date)
        .bind(reenrollment_deadline)
        .fetch_one(&self.pool)
        .await?;

        Ok(period)
    }

    pub async fn find_by_id(&self, id: i32) -> AppResult<Option<Period>> {
        let period = sqlx::query_as::<_, Period>(
            r#"
            SELECT * FROM periods WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(period)
    }

    pub async fn find_by_school(
        &self,
        school_id: i32,
        limit: i64,
        offset: i64,
        status: Option<String>,
        academic_year: Option<String>,
        level: Option<String>,
    ) -> AppResult<Vec<Period>> {
        let mut query = String::from("SELECT * FROM periods WHERE school_id = $1");
        let mut param_count = 1;

        if status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
        }

        if academic_year.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND academic_year = ${}", param_count));
        }

        if level.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND level = ${}", param_count));
        }

        query.push_str(" ORDER BY created_at DESC");
        query.push_str(&format!(" LIMIT ${} OFFSET ${}", param_count + 1, param_count + 2));

        let mut query_builder = sqlx::query_as::<_, Period>(&query).bind(school_id);

        if let Some(s) = status {
            query_builder = query_builder.bind(s);
        }
        if let Some(ay) = academic_year {
            query_builder = query_builder.bind(ay);
        }
        if let Some(l) = level {
            query_builder = query_builder.bind(l);
        }

        let periods = query_builder
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok(periods)
    }

    pub async fn count_by_school(
        &self,
        school_id: i32,
        status: Option<String>,
        academic_year: Option<String>,
        level: Option<String>,
    ) -> AppResult<i64> {
        let mut query = String::from("SELECT COUNT(*) FROM periods WHERE school_id = $1");
        let mut param_count = 1;

        if status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
        }

        if academic_year.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND academic_year = ${}", param_count));
        }

        if level.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND level = ${}", param_count));
        }

        let mut query_builder = sqlx::query_scalar::<_, i64>(&query).bind(school_id);

        if let Some(s) = status {
            query_builder = query_builder.bind(s);
        }
        if let Some(ay) = academic_year {
            query_builder = query_builder.bind(ay);
        }
        if let Some(l) = level {
            query_builder = query_builder.bind(l);
        }

        let count = query_builder.fetch_one(&self.pool).await?;

        Ok(count)
    }

    pub async fn find_active_by_school_and_level(
        &self,
        school_id: i32,
        academic_year: &str,
        level: &str,
    ) -> AppResult<Option<Period>> {
        let period = sqlx::query_as::<_, Period>(
            r#"
            SELECT * FROM periods 
            WHERE school_id = $1 AND academic_year = $2 AND level = $3 AND status = 'active'
            LIMIT 1
            "#,
        )
        .bind(school_id)
        .bind(academic_year)
        .bind(level)
        .fetch_optional(&self.pool)
        .await?;

        Ok(period)
    }

    pub async fn update_period(
        &self,
        id: i32,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
        announcement_date: Option<NaiveDate>,
        reenrollment_deadline: Option<NaiveDate>,
    ) -> AppResult<Period> {
        let period = sqlx::query_as::<_, Period>(
            r#"
            UPDATE periods 
            SET start_date = COALESCE($2, start_date),
                end_date = COALESCE($3, end_date),
                announcement_date = COALESCE($4, announcement_date),
                reenrollment_deadline = COALESCE($5, reenrollment_deadline),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(start_date)
        .bind(end_date)
        .bind(announcement_date)
        .bind(reenrollment_deadline)
        .fetch_one(&self.pool)
        .await?;

        Ok(period)
    }

    pub async fn update_status(&self, id: i32, status: &str) -> AppResult<Period> {
        let period = sqlx::query_as::<_, Period>(
            r#"
            UPDATE periods 
            SET status = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        Ok(period)
    }

    pub async fn delete_period(&self, id: i32) -> AppResult<()> {
        sqlx::query("DELETE FROM periods WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Registration Path methods
    pub async fn create_path(
        &self,
        period_id: i32,
        path_type: &str,
        name: &str,
        quota: i32,
        description: Option<&str>,
        scoring_config: serde_json::Value,
    ) -> AppResult<RegistrationPath> {
        let path = sqlx::query_as::<_, RegistrationPath>(
            r#"
            INSERT INTO registration_paths (period_id, path_type, name, quota, description, scoring_config)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(period_id)
        .bind(path_type)
        .bind(name)
        .bind(quota)
        .bind(description)
        .bind(scoring_config)
        .fetch_one(&self.pool)
        .await?;

        Ok(path)
    }

    pub async fn find_paths_by_period(&self, period_id: i32) -> AppResult<Vec<RegistrationPath>> {
        let paths = sqlx::query_as::<_, RegistrationPath>(
            r#"
            SELECT * FROM registration_paths WHERE period_id = $1 ORDER BY id
            "#,
        )
        .bind(period_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(paths)
    }

    pub async fn find_path_by_id(&self, id: i32) -> AppResult<Option<RegistrationPath>> {
        let path = sqlx::query_as::<_, RegistrationPath>(
            r#"
            SELECT * FROM registration_paths WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(path)
    }

    pub async fn update_path(
        &self,
        id: i32,
        name: Option<&str>,
        quota: Option<i32>,
        description: Option<&str>,
        scoring_config: Option<serde_json::Value>,
    ) -> AppResult<RegistrationPath> {
        let path = sqlx::query_as::<_, RegistrationPath>(
            r#"
            UPDATE registration_paths 
            SET name = COALESCE($2, name),
                quota = COALESCE($3, quota),
                description = COALESCE($4, description),
                scoring_config = COALESCE($5, scoring_config),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(quota)
        .bind(description)
        .bind(scoring_config)
        .fetch_one(&self.pool)
        .await?;

        Ok(path)
    }

    pub async fn delete_path(&self, id: i32) -> AppResult<()> {
        sqlx::query("DELETE FROM registration_paths WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
