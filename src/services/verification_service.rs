use crate::models::registration::Registration;
use crate::repositories::registration_repo::RegistrationRepository;
use crate::utils::error::{AppError, AppResult};

pub struct VerificationService {
    registration_repo: RegistrationRepository,
}

impl VerificationService {
    pub fn new(registration_repo: RegistrationRepository) -> Self {
        Self { registration_repo }
    }

    pub async fn get_pending_verifications(
        &self,
        school_id: i32,
        page: i64,
        page_size: i64,
        period_id: Option<i32>,
        path_id: Option<i32>,
    ) -> AppResult<(Vec<Registration>, i64)> {
        let offset = (page - 1) * page_size;

        // Get submitted registrations
        let registrations = self
            .registration_repo
            .find_by_school(
                school_id,
                page_size,
                offset,
                Some("submitted".to_string()),
                period_id,
                path_id,
            )
            .await?;

        let total = self
            .registration_repo
            .count_by_school(school_id, Some("submitted".to_string()), period_id, path_id)
            .await?;

        Ok((registrations, total))
    }

    pub async fn verify_registration(&self, id: i32, admin_id: i32) -> AppResult<Registration> {
        // Check if registration exists
        let registration = self
            .registration_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Registration not found".to_string()))?;

        // Only allow verification if status is submitted
        if registration.status != "submitted" {
            return Err(AppError::Validation(
                "Can only verify registrations in submitted status".to_string(),
            ));
        }

        // Check if all documents are uploaded
        let documents = self
            .registration_repo
            .find_documents_by_registration(id)
            .await?;

        if documents.is_empty() {
            return Err(AppError::Validation(
                "Cannot verify registration without documents".to_string(),
            ));
        }

        // Update status to verified
        let verified_registration = self
            .registration_repo
            .update_status(id, "verified", None)
            .await?;

        // TODO: Send approval notification email
        tracing::info!(
            "Registration {} verified by admin {}",
            id,
            admin_id
        );

        // TODO: Log to audit_logs

        Ok(verified_registration)
    }

    pub async fn reject_registration(
        &self,
        id: i32,
        reason: String,
        admin_id: i32,
    ) -> AppResult<Registration> {
        // Check if registration exists
        let registration = self
            .registration_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Registration not found".to_string()))?;

        // Only allow rejection if status is submitted
        if registration.status != "submitted" {
            return Err(AppError::Validation(
                "Can only reject registrations in submitted status".to_string(),
            ));
        }

        // Validate reason
        if reason.trim().is_empty() {
            return Err(AppError::Validation(
                "Rejection reason is required".to_string(),
            ));
        }

        // Update status to rejected with reason
        let rejected_registration = self
            .registration_repo
            .update_status(id, "rejected", Some(&reason))
            .await?;

        // TODO: Send rejection notification email with reason
        tracing::info!(
            "Registration {} rejected by admin {} with reason: {}",
            id,
            admin_id,
            reason
        );

        // TODO: Log to audit_logs

        Ok(rejected_registration)
    }

    pub async fn verify_document(
        &self,
        document_id: i32,
        verification_status: String,
        verification_notes: Option<String>,
        admin_id: i32,
    ) -> AppResult<()> {
        // Validate verification status
        if verification_status != "approved" && verification_status != "rejected" {
            return Err(AppError::Validation(
                "Verification status must be 'approved' or 'rejected'".to_string(),
            ));
        }

        // Check if document exists
        let document = self
            .registration_repo
            .find_document_by_id(document_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        // Update document verification status
        self.registration_repo
            .update_document_verification(
                document_id,
                &verification_status,
                verification_notes.as_deref(),
            )
            .await?;

        tracing::info!(
            "Document {} verification status updated to {} by admin {}",
            document_id,
            verification_status,
            admin_id
        );

        // TODO: Log to audit_logs

        Ok(())
    }

    pub async fn get_verification_statistics(
        &self,
        school_id: i32,
        period_id: Option<i32>,
    ) -> AppResult<VerificationStats> {
        let submitted_count = self
            .registration_repo
            .count_by_school(school_id, Some("submitted".to_string()), period_id, None)
            .await?;

        let verified_count = self
            .registration_repo
            .count_by_school(school_id, Some("verified".to_string()), period_id, None)
            .await?;

        let rejected_count = self
            .registration_repo
            .count_by_school(school_id, Some("rejected".to_string()), period_id, None)
            .await?;

        let total_count = self
            .registration_repo
            .count_by_school(school_id, None, period_id, None)
            .await?;

        Ok(VerificationStats {
            total: total_count,
            submitted: submitted_count,
            verified: verified_count,
            rejected: rejected_count,
            pending: submitted_count,
        })
    }
}

/// Statistik verifikasi
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct VerificationStats {
    /// Total pendaftaran
    #[schema(example = 150)]
    pub total: i64,
    
    /// Total yang sudah disubmit
    #[schema(example = 120)]
    pub submitted: i64,
    
    /// Total yang sudah diverifikasi
    #[schema(example = 80)]
    pub verified: i64,
    
    /// Total yang ditolak
    #[schema(example = 10)]
    pub rejected: i64,
    
    /// Total yang menunggu verifikasi
    #[schema(example = 30)]
    pub pending: i64,
}
