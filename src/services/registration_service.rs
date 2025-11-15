use chrono::{DateTime, NaiveDate, NaiveTime, Utc};

use crate::models::registration::{Document, Registration};
use crate::repositories::period_repo::PeriodRepository;
use crate::repositories::registration_repo::RegistrationRepository;
use crate::utils::error::{AppError, AppResult};

pub struct RegistrationService {
    registration_repo: RegistrationRepository,
    period_repo: PeriodRepository,
}

impl RegistrationService {
    pub fn new(registration_repo: RegistrationRepository, period_repo: PeriodRepository) -> Self {
        Self {
            registration_repo,
            period_repo,
        }
    }

    pub async fn create_registration(
        &self,
        school_id: i32,
        user_id: i32,
        period_id: i32,
        path_id: i32,
        student_nisn: String,
        student_name: String,
        student_gender: String,
        student_birth_place: String,
        student_birth_date: NaiveDate,
        student_religion: String,
        student_address: String,
        student_phone: Option<String>,
        student_email: Option<String>,
        parent_name: String,
        parent_nik: String,
        parent_phone: String,
        parent_occupation: Option<String>,
        parent_income: Option<String>,
        previous_school_name: Option<String>,
        previous_school_npsn: Option<String>,
        previous_school_address: Option<String>,
        path_data: serde_json::Value,
    ) -> AppResult<Registration> {
        // Validate period exists and is active
        let period = self
            .period_repo
            .find_by_id(period_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Period not found".to_string()))?;

        if period.status != "active" {
            return Err(AppError::Validation(
                "Period is not active. Registration is closed.".to_string(),
            ));
        }

        // Validate path exists and belongs to period
        let path = self
            .period_repo
            .find_path_by_id(path_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Registration path not found".to_string()))?;

        if path.period_id != period_id {
            return Err(AppError::Validation(
                "Registration path does not belong to this period".to_string(),
            ));
        }

        // Validate NISN format (16 digits)
        if student_nisn.len() != 10 {
            return Err(AppError::Validation(
                "NISN must be 10 digits".to_string(),
            ));
        }

        // Validate NIK format (16 digits)
        if parent_nik.len() != 16 {
            return Err(AppError::Validation(
                "NIK must be 16 digits".to_string(),
            ));
        }

        // Convert NaiveDate to DateTime<Utc>
        let student_birth_datetime = student_birth_date
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .and_utc();

        // Create registration
        let registration = self
            .registration_repo
            .create_registration(
                school_id,
                user_id,
                period_id,
                path_id,
                &student_nisn,
                &student_name,
                &student_gender,
                &student_birth_place,
                student_birth_datetime,
                &student_religion,
                &student_address,
                student_phone.as_deref(),
                student_email.as_deref(),
                &parent_name,
                &parent_nik,
                &parent_phone,
                parent_occupation.as_deref(),
                parent_income.as_deref(),
                previous_school_name.as_deref(),
                previous_school_npsn.as_deref(),
                previous_school_address.as_deref(),
                path_data,
            )
            .await?;

        Ok(registration)
    }

    pub async fn get_registration(&self, id: i32) -> AppResult<Registration> {
        self.registration_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Registration not found".to_string()))
    }

    pub async fn list_registrations_by_user(
        &self,
        user_id: i32,
        page: i64,
        page_size: i64,
    ) -> AppResult<Vec<Registration>> {
        let offset = (page - 1) * page_size;

        let registrations = self
            .registration_repo
            .find_by_user(user_id, page_size, offset)
            .await?;

        Ok(registrations)
    }

    pub async fn list_registrations_by_school(
        &self,
        school_id: i32,
        page: i64,
        page_size: i64,
        status: Option<String>,
        period_id: Option<i32>,
        path_id: Option<i32>,
    ) -> AppResult<(Vec<Registration>, i64)> {
        let offset = (page - 1) * page_size;

        let registrations = self
            .registration_repo
            .find_by_school(school_id, page_size, offset, status.clone(), period_id, path_id)
            .await?;

        let total = self
            .registration_repo
            .count_by_school(school_id, status, period_id, path_id)
            .await?;

        Ok((registrations, total))
    }

    pub async fn update_registration(
        &self,
        id: i32,
        student_name: Option<String>,
        student_gender: Option<String>,
        student_birth_place: Option<String>,
        student_birth_date: Option<NaiveDate>,
        student_religion: Option<String>,
        student_address: Option<String>,
        student_phone: Option<String>,
        student_email: Option<String>,
        parent_name: Option<String>,
        parent_nik: Option<String>,
        parent_phone: Option<String>,
        parent_occupation: Option<String>,
        parent_income: Option<String>,
        path_data: Option<serde_json::Value>,
    ) -> AppResult<Registration> {
        // Check if registration exists
        let registration = self.get_registration(id).await?;

        // Only allow updates if status is draft
        if registration.status != "draft" {
            return Err(AppError::Validation(
                "Can only update registrations in draft status".to_string(),
            ));
        }

        // Convert NaiveDate to DateTime<Utc> if provided
        let student_birth_datetime = student_birth_date.map(|date| {
            date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                .and_utc()
        });

        // Update registration
        let updated_registration = self
            .registration_repo
            .update_registration(
                id,
                student_name.as_deref(),
                student_gender.as_deref(),
                student_birth_place.as_deref(),
                student_birth_datetime,
                student_religion.as_deref(),
                student_address.as_deref(),
                student_phone.as_deref(),
                student_email.as_deref(),
                parent_name.as_deref(),
                parent_nik.as_deref(),
                parent_phone.as_deref(),
                parent_occupation.as_deref(),
                parent_income.as_deref(),
                path_data,
            )
            .await?;

        Ok(updated_registration)
    }

    pub async fn submit_registration(&self, id: i32) -> AppResult<Registration> {
        // Check if registration exists
        let registration = self.get_registration(id).await?;

        // Only allow submission if status is draft
        if registration.status != "draft" {
            return Err(AppError::Validation(
                "Can only submit registrations in draft status".to_string(),
            ));
        }

        // Validate completeness - check if required documents are uploaded
        let documents = self
            .registration_repo
            .find_documents_by_registration(id)
            .await?;

        if documents.is_empty() {
            return Err(AppError::Validation(
                "Please upload required documents before submitting".to_string(),
            ));
        }

        // Generate registration number
        let registration_number = self
            .registration_repo
            .generate_registration_number(registration.school_id, registration.period_id)
            .await?;

        // Set registration number
        let _ = self
            .registration_repo
            .set_registration_number(id, &registration_number)
            .await?;

        // Update status to submitted
        let submitted_registration = self
            .registration_repo
            .update_status(id, "submitted", None)
            .await?;

        // TODO: Send notification email

        Ok(submitted_registration)
    }

    // Document methods
    pub async fn upload_document(
        &self,
        registration_id: i32,
        document_type: String,
        file_url: String,
        file_name: String,
        file_size: i64,
        mime_type: String,
    ) -> AppResult<Document> {
        // Check if registration exists
        let registration = self.get_registration(registration_id).await?;

        // Only allow document upload if status is draft
        if registration.status != "draft" {
            return Err(AppError::Validation(
                "Can only upload documents for registrations in draft status".to_string(),
            ));
        }

        // Validate file size (max 2MB)
        if file_size > 2 * 1024 * 1024 {
            return Err(AppError::Validation(
                "File size must not exceed 2MB".to_string(),
            ));
        }

        // Validate mime type
        let allowed_types = vec!["image/jpeg", "image/png", "image/jpg", "application/pdf"];
        if !allowed_types.contains(&mime_type.as_str()) {
            return Err(AppError::Validation(
                "File type must be JPEG, PNG, or PDF".to_string(),
            ));
        }

        // Create document
        let document = self
            .registration_repo
            .create_document(
                registration_id,
                &document_type,
                &file_url,
                &file_name,
                file_size,
                &mime_type,
            )
            .await?;

        Ok(document)
    }

    pub async fn list_documents(&self, registration_id: i32) -> AppResult<Vec<Document>> {
        // Check if registration exists
        let _ = self.get_registration(registration_id).await?;

        let documents = self
            .registration_repo
            .find_documents_by_registration(registration_id)
            .await?;

        Ok(documents)
    }

    pub async fn delete_document(&self, id: i32, user_id: i32) -> AppResult<()> {
        // Check if document exists
        let document = self
            .registration_repo
            .find_document_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        // Check if registration belongs to user
        let registration = self.get_registration(document.registration_id).await?;
        if registration.user_id != user_id {
            return Err(AppError::Forbidden(
                "You don't have permission to delete this document".to_string(),
            ));
        }

        // Only allow deletion if registration is in draft status
        if registration.status != "draft" {
            return Err(AppError::Validation(
                "Can only delete documents for registrations in draft status".to_string(),
            ));
        }

        // Delete document
        self.registration_repo.delete_document(id).await?;

        // TODO: Delete file from storage

        Ok(())
    }
}
