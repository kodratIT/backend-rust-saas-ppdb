use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::models::registration::{Document, Registration};
use crate::utils::error::AppResult;

pub struct RegistrationRepository {
    pool: PgPool,
}

impl RegistrationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_registration(
        &self,
        school_id: i32,
        user_id: i32,
        period_id: i32,
        path_id: i32,
        student_nisn: &str,
        student_name: &str,
        student_gender: &str,
        student_birth_place: &str,
        student_birth_date: DateTime<Utc>,
        student_religion: &str,
        student_address: &str,
        student_phone: Option<&str>,
        student_email: Option<&str>,
        parent_name: &str,
        parent_nik: &str,
        parent_phone: &str,
        parent_occupation: Option<&str>,
        parent_income: Option<&str>,
        previous_school_name: Option<&str>,
        previous_school_npsn: Option<&str>,
        previous_school_address: Option<&str>,
        path_data: serde_json::Value,
    ) -> AppResult<Registration> {
        let registration = sqlx::query_as::<_, Registration>(
            r#"
            INSERT INTO registrations (
                school_id, user_id, period_id, path_id,
                student_nisn, student_name, student_gender, student_birth_place, student_birth_date,
                student_religion, student_address, student_phone, student_email,
                parent_name, parent_nik, parent_phone, parent_occupation, parent_income,
                previous_school_name, previous_school_npsn, previous_school_address,
                path_data, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, 'draft')
            RETURNING *
            "#,
        )
        .bind(school_id)
        .bind(user_id)
        .bind(period_id)
        .bind(path_id)
        .bind(student_nisn)
        .bind(student_name)
        .bind(student_gender)
        .bind(student_birth_place)
        .bind(student_birth_date)
        .bind(student_religion)
        .bind(student_address)
        .bind(student_phone)
        .bind(student_email)
        .bind(parent_name)
        .bind(parent_nik)
        .bind(parent_phone)
        .bind(parent_occupation)
        .bind(parent_income)
        .bind(previous_school_name)
        .bind(previous_school_npsn)
        .bind(previous_school_address)
        .bind(path_data)
        .fetch_one(&self.pool)
        .await?;

        Ok(registration)
    }

    pub async fn find_by_id(&self, id: i32) -> AppResult<Option<Registration>> {
        let registration = sqlx::query_as::<_, Registration>(
            r#"
            SELECT * FROM registrations WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(registration)
    }

    pub async fn find_by_user(
        &self,
        user_id: i32,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<Registration>> {
        let registrations = sqlx::query_as::<_, Registration>(
            r#"
            SELECT * FROM registrations 
            WHERE user_id = $1 
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(registrations)
    }

    pub async fn find_by_school(
        &self,
        school_id: i32,
        limit: i64,
        offset: i64,
        status: Option<String>,
        period_id: Option<i32>,
        path_id: Option<i32>,
    ) -> AppResult<Vec<Registration>> {
        let mut query = String::from("SELECT * FROM registrations WHERE school_id = $1");
        let mut param_count = 1;

        if status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
        }

        if period_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND period_id = ${}", param_count));
        }

        if path_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND path_id = ${}", param_count));
        }

        query.push_str(" ORDER BY created_at DESC");
        query.push_str(&format!(" LIMIT ${} OFFSET ${}", param_count + 1, param_count + 2));

        let mut query_builder = sqlx::query_as::<_, Registration>(&query).bind(school_id);

        if let Some(s) = status {
            query_builder = query_builder.bind(s);
        }
        if let Some(p) = period_id {
            query_builder = query_builder.bind(p);
        }
        if let Some(pa) = path_id {
            query_builder = query_builder.bind(pa);
        }

        let registrations = query_builder
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok(registrations)
    }

    pub async fn count_by_school(
        &self,
        school_id: i32,
        status: Option<String>,
        period_id: Option<i32>,
        path_id: Option<i32>,
    ) -> AppResult<i64> {
        let mut query = String::from("SELECT COUNT(*) FROM registrations WHERE school_id = $1");
        let mut param_count = 1;

        if status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
        }

        if period_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND period_id = ${}", param_count));
        }

        if path_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND path_id = ${}", param_count));
        }

        let mut query_builder = sqlx::query_scalar::<_, i64>(&query).bind(school_id);

        if let Some(s) = status {
            query_builder = query_builder.bind(s);
        }
        if let Some(p) = period_id {
            query_builder = query_builder.bind(p);
        }
        if let Some(pa) = path_id {
            query_builder = query_builder.bind(pa);
        }

        let count = query_builder.fetch_one(&self.pool).await?;

        Ok(count)
    }

    pub async fn update_registration(
        &self,
        id: i32,
        student_name: Option<&str>,
        student_gender: Option<&str>,
        student_birth_place: Option<&str>,
        student_birth_date: Option<DateTime<Utc>>,
        student_religion: Option<&str>,
        student_address: Option<&str>,
        student_phone: Option<&str>,
        student_email: Option<&str>,
        parent_name: Option<&str>,
        parent_nik: Option<&str>,
        parent_phone: Option<&str>,
        parent_occupation: Option<&str>,
        parent_income: Option<&str>,
        path_data: Option<serde_json::Value>,
    ) -> AppResult<Registration> {
        let registration = sqlx::query_as::<_, Registration>(
            r#"
            UPDATE registrations 
            SET student_name = COALESCE($2, student_name),
                student_gender = COALESCE($3, student_gender),
                student_birth_place = COALESCE($4, student_birth_place),
                student_birth_date = COALESCE($5, student_birth_date),
                student_religion = COALESCE($6, student_religion),
                student_address = COALESCE($7, student_address),
                student_phone = COALESCE($8, student_phone),
                student_email = COALESCE($9, student_email),
                parent_name = COALESCE($10, parent_name),
                parent_nik = COALESCE($11, parent_nik),
                parent_phone = COALESCE($12, parent_phone),
                parent_occupation = COALESCE($13, parent_occupation),
                parent_income = COALESCE($14, parent_income),
                path_data = COALESCE($15, path_data),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(student_name)
        .bind(student_gender)
        .bind(student_birth_place)
        .bind(student_birth_date)
        .bind(student_religion)
        .bind(student_address)
        .bind(student_phone)
        .bind(student_email)
        .bind(parent_name)
        .bind(parent_nik)
        .bind(parent_phone)
        .bind(parent_occupation)
        .bind(parent_income)
        .bind(path_data)
        .fetch_one(&self.pool)
        .await?;

        Ok(registration)
    }

    pub async fn update_status(
        &self,
        id: i32,
        status: &str,
        rejection_reason: Option<&str>,
    ) -> AppResult<Registration> {
        let registration = sqlx::query_as::<_, Registration>(
            r#"
            UPDATE registrations 
            SET status = $2, 
                rejection_reason = $3,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(rejection_reason)
        .fetch_one(&self.pool)
        .await?;

        Ok(registration)
    }

    pub async fn set_registration_number(&self, id: i32, registration_number: &str) -> AppResult<Registration> {
        let registration = sqlx::query_as::<_, Registration>(
            r#"
            UPDATE registrations 
            SET registration_number = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(registration_number)
        .fetch_one(&self.pool)
        .await?;

        Ok(registration)
    }

    pub async fn generate_registration_number(&self, school_id: i32, period_id: i32) -> AppResult<String> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM registrations 
            WHERE school_id = $1 AND period_id = $2 AND registration_number IS NOT NULL
            "#,
        )
        .bind(school_id)
        .bind(period_id)
        .fetch_one(&self.pool)
        .await?;

        let number = count + 1;
        let registration_number = format!("REG-{}-{}-{:05}", school_id, period_id, number);

        Ok(registration_number)
    }

    // Document methods
    pub async fn create_document(
        &self,
        registration_id: i32,
        document_type: &str,
        file_url: &str,
        file_name: &str,
        file_size: i64,
        mime_type: &str,
    ) -> AppResult<Document> {
        let document = sqlx::query_as::<_, Document>(
            r#"
            INSERT INTO documents (registration_id, document_type, file_url, file_name, file_size, mime_type, verification_status)
            VALUES ($1, $2, $3, $4, $5, $6, 'pending')
            RETURNING *
            "#,
        )
        .bind(registration_id)
        .bind(document_type)
        .bind(file_url)
        .bind(file_name)
        .bind(file_size)
        .bind(mime_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(document)
    }

    pub async fn find_documents_by_registration(&self, registration_id: i32) -> AppResult<Vec<Document>> {
        let documents = sqlx::query_as::<_, Document>(
            r#"
            SELECT * FROM documents WHERE registration_id = $1 ORDER BY created_at
            "#,
        )
        .bind(registration_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(documents)
    }

    pub async fn find_document_by_id(&self, id: i32) -> AppResult<Option<Document>> {
        let document = sqlx::query_as::<_, Document>(
            r#"
            SELECT * FROM documents WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(document)
    }

    pub async fn update_document_verification(
        &self,
        id: i32,
        verification_status: &str,
        verification_notes: Option<&str>,
    ) -> AppResult<Document> {
        let document = sqlx::query_as::<_, Document>(
            r#"
            UPDATE documents 
            SET verification_status = $2, 
                verification_notes = $3,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(verification_status)
        .bind(verification_notes)
        .fetch_one(&self.pool)
        .await?;

        Ok(document)
    }

    pub async fn delete_document(&self, id: i32) -> AppResult<()> {
        sqlx::query("DELETE FROM documents WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
