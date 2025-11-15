use chrono::Utc;

use crate::models::registration::Registration;
use crate::repositories::period_repo::PeriodRepository;
use crate::repositories::registration_repo::RegistrationRepository;
use crate::utils::error::{AppError, AppResult};

pub struct AnnouncementService {
    registration_repo: RegistrationRepository,
    period_repo: PeriodRepository,
}

impl AnnouncementService {
    pub fn new(
        registration_repo: RegistrationRepository,
        period_repo: PeriodRepository,
    ) -> Self {
        Self {
            registration_repo,
            period_repo,
        }
    }

    /// Run selection process for a period
    /// Select top N registrations per path (N = quota)
    /// Update status to Accepted for selected, Rejected for non-selected
    pub async fn run_selection(&self, period_id: i32, admin_id: i32) -> AppResult<SelectionResult> {
        // Check if period exists
        let period = self
            .period_repo
            .find_by_id(period_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Period not found".to_string()))?;

        // Check if period is active
        if period.status != "active" {
            return Err(AppError::Validation(
                "Can only run selection for active periods".to_string(),
            ));
        }

        // Get all paths for this period
        let paths = self.period_repo.find_paths_by_period(period_id).await?;

        let mut total_accepted = 0;
        let mut total_rejected = 0;

        // Run selection for each path
        for path in paths {
            // Get all verified registrations with rankings for this path
            let registrations = sqlx::query_as::<_, Registration>(
                r#"
                SELECT * FROM registrations 
                WHERE period_id = $1 
                  AND path_id = $2 
                  AND status = 'verified'
                  AND ranking IS NOT NULL
                ORDER BY ranking ASC
                "#,
            )
            .bind(period_id)
            .bind(path.id)
            .fetch_all(&self.registration_repo.pool)
            .await?;

            // Select top N (N = quota)
            for (index, registration) in registrations.iter().enumerate() {
                let new_status = if (index as i32) < path.quota {
                    "accepted"
                } else {
                    "rejected"
                };

                // Update status
                self.registration_repo
                    .update_status(
                        registration.id,
                        new_status,
                        if new_status == "rejected" {
                            Some("Quota penuh. Anda berada di luar kuota yang tersedia.")
                        } else {
                            None
                        },
                    )
                    .await?;

                if new_status == "accepted" {
                    total_accepted += 1;
                } else {
                    total_rejected += 1;
                }
            }
        }

        tracing::info!(
            "Selection completed for period {} by admin {}. Accepted: {}, Rejected: {}",
            period_id,
            admin_id,
            total_accepted,
            total_rejected
        );

        // TODO: Log to audit_logs

        Ok(SelectionResult {
            total_accepted,
            total_rejected,
        })
    }

    /// Announce results - send notifications to all registrations
    pub async fn announce_results(&self, period_id: i32, admin_id: i32) -> AppResult<AnnouncementResult> {
        // Check if period exists
        let period = self
            .period_repo
            .find_by_id(period_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Period not found".to_string()))?;

        // Check if selection has been run (there should be accepted/rejected registrations)
        let accepted_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM registrations 
            WHERE period_id = $1 AND status = 'accepted'
            "#,
        )
        .bind(period_id)
        .fetch_one(&self.registration_repo.pool)
        .await?;

        if accepted_count == 0 {
            return Err(AppError::Validation(
                "No accepted registrations found. Please run selection first.".to_string(),
            ));
        }

        // Update announcement_date in period
        sqlx::query(
            r#"
            UPDATE periods 
            SET announcement_date = $2, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(period_id)
        .bind(Utc::now())
        .execute(&self.registration_repo.pool)
        .await?;

        // Get all accepted registrations
        let accepted_registrations = sqlx::query_as::<_, Registration>(
            r#"
            SELECT * FROM registrations 
            WHERE period_id = $1 AND status = 'accepted'
            "#,
        )
        .bind(period_id)
        .fetch_all(&self.registration_repo.pool)
        .await?;

        // Get all rejected registrations
        let rejected_registrations = sqlx::query_as::<_, Registration>(
            r#"
            SELECT * FROM registrations 
            WHERE period_id = $1 AND status = 'rejected'
            "#,
        )
        .bind(period_id)
        .fetch_all(&self.registration_repo.pool)
        .await?;

        // TODO: Send acceptance emails
        for registration in &accepted_registrations {
            tracing::info!(
                "Sending acceptance notification to registration {}",
                registration.id
            );
            // TODO: Implement email sending
        }

        // TODO: Send rejection emails
        for registration in &rejected_registrations {
            tracing::info!(
                "Sending rejection notification to registration {}",
                registration.id
            );
            // TODO: Implement email sending
        }

        tracing::info!(
            "Results announced for period {} by admin {}. Notifications sent: {} accepted, {} rejected",
            period_id,
            admin_id,
            accepted_registrations.len(),
            rejected_registrations.len()
        );

        // TODO: Log to audit_logs

        Ok(AnnouncementResult {
            total_notified: (accepted_registrations.len() + rejected_registrations.len()) as i32,
            accepted_notified: accepted_registrations.len() as i32,
            rejected_notified: rejected_registrations.len() as i32,
        })
    }

    /// Check registration result by registration number and NISN (public endpoint)
    pub async fn check_result(
        &self,
        registration_number: String,
        student_nisn: String,
    ) -> AppResult<ResultCheckResponse> {
        // Find registration by registration_number and student_nisn
        let registration = sqlx::query_as::<_, Registration>(
            r#"
            SELECT * FROM registrations 
            WHERE registration_number = $1 AND student_nisn = $2
            "#,
        )
        .bind(&registration_number)
        .bind(&student_nisn)
        .fetch_optional(&self.registration_repo.pool)
        .await?
        .ok_or_else(|| {
            AppError::NotFound("Registration not found or NISN does not match".to_string())
        })?;

        // Check if results have been announced
        let period = self
            .period_repo
            .find_by_id(registration.period_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Period not found".to_string()))?;

        if period.announcement_date.is_none() {
            return Err(AppError::Validation(
                "Results have not been announced yet".to_string(),
            ));
        }

        // Get path info
        let path = self
            .period_repo
            .find_path_by_id(registration.path_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Registration path not found".to_string()))?;

        Ok(ResultCheckResponse {
            registration_number: registration.registration_number.unwrap_or_default(),
            student_name: registration.student_name,
            student_nisn: registration.student_nisn,
            path_name: path.name,
            selection_score: registration.selection_score,
            ranking: registration.ranking,
            status: registration.status,
            rejection_reason: registration.rejection_reason,
            announcement_date: period.announcement_date,
            reenrollment_deadline: period.reenrollment_deadline,
        })
    }

    /// Get selection summary for a period
    pub async fn get_selection_summary(&self, period_id: i32) -> AppResult<SelectionSummary> {
        // Check if period exists
        let _ = self
            .period_repo
            .find_by_id(period_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Period not found".to_string()))?;

        // Get counts by status
        let verified_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM registrations WHERE period_id = $1 AND status = 'verified'",
        )
        .bind(period_id)
        .fetch_one(&self.registration_repo.pool)
        .await?;

        let accepted_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM registrations WHERE period_id = $1 AND status = 'accepted'",
        )
        .bind(period_id)
        .fetch_one(&self.registration_repo.pool)
        .await?;

        let rejected_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM registrations WHERE period_id = $1 AND status = 'rejected'",
        )
        .bind(period_id)
        .fetch_one(&self.registration_repo.pool)
        .await?;

        // Get path summaries
        let paths = self.period_repo.find_paths_by_period(period_id).await?;
        let mut path_summaries = Vec::new();

        for path in paths {
            let path_accepted: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM registrations WHERE period_id = $1 AND path_id = $2 AND status = 'accepted'",
            )
            .bind(period_id)
            .bind(path.id)
            .fetch_one(&self.registration_repo.pool)
            .await?;

            let path_rejected: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM registrations WHERE period_id = $1 AND path_id = $2 AND status = 'rejected'",
            )
            .bind(period_id)
            .bind(path.id)
            .fetch_one(&self.registration_repo.pool)
            .await?;

            path_summaries.push(PathSelectionSummary {
                path_id: path.id,
                path_name: path.name,
                quota: path.quota,
                accepted: path_accepted,
                rejected: path_rejected,
                remaining_quota: (path.quota as i64 - path_accepted).max(0),
            });
        }

        Ok(SelectionSummary {
            period_id,
            verified: verified_count,
            accepted: accepted_count,
            rejected: rejected_count,
            paths: path_summaries,
        })
    }
}

/// Hasil proses seleksi
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct SelectionResult {
    /// Total yang diterima
    #[schema(example = 100)]
    pub total_accepted: i32,
    
    /// Total yang ditolak
    #[schema(example = 50)]
    pub total_rejected: i32,
}

/// Hasil pengumuman
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AnnouncementResult {
    /// Total notifikasi terkirim
    #[schema(example = 150)]
    pub total_notified: i32,
    
    /// Notifikasi diterima terkirim
    #[schema(example = 100)]
    pub accepted_notified: i32,
    
    /// Notifikasi ditolak terkirim
    #[schema(example = 50)]
    pub rejected_notified: i32,
}

/// Response cek hasil seleksi
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ResultCheckResponse {
    /// Nomor pendaftaran
    #[schema(example = "PPDB-2024-001")]
    pub registration_number: String,
    
    /// Nama siswa
    #[schema(example = "Ahmad Fauzi")]
    pub student_name: String,
    
    /// NISN siswa
    #[schema(example = "0012345678")]
    pub student_nisn: String,
    
    /// Nama jalur
    #[schema(example = "Jalur Zonasi")]
    pub path_name: String,
    
    /// Skor seleksi
    #[schema(example = 85.5)]
    pub selection_score: Option<f64>,
    
    /// Peringkat
    #[schema(example = 10)]
    pub ranking: Option<i32>,
    
    /// Status (accepted/rejected)
    #[schema(example = "accepted")]
    pub status: String,
    
    /// Alasan penolakan
    #[schema(example = "Kuota penuh")]
    pub rejection_reason: Option<String>,
    
    /// Tanggal pengumuman
    #[schema(value_type = Option<String>, example = "2024-08-01")]
    pub announcement_date: Option<chrono::NaiveDate>,
    
    /// Batas waktu daftar ulang
    #[schema(value_type = Option<String>, example = "2024-08-15")]
    pub reenrollment_deadline: Option<chrono::NaiveDate>,
}

/// Ringkasan seleksi
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct SelectionSummary {
    /// ID periode
    #[schema(example = 1)]
    pub period_id: i32,
    
    /// Total terverifikasi
    #[schema(example = 150)]
    pub verified: i64,
    
    /// Total diterima
    #[schema(example = 100)]
    pub accepted: i64,
    
    /// Total ditolak
    #[schema(example = 50)]
    pub rejected: i64,
    
    /// Ringkasan per jalur
    pub paths: Vec<PathSelectionSummary>,
}

/// Ringkasan seleksi per jalur
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PathSelectionSummary {
    /// ID jalur
    #[schema(example = 1)]
    pub path_id: i32,
    
    /// Nama jalur
    #[schema(example = "Jalur Zonasi")]
    pub path_name: String,
    
    /// Kuota jalur
    #[schema(example = 100)]
    pub quota: i32,
    
    /// Total diterima
    #[schema(example = 80)]
    pub accepted: i64,
    
    /// Total ditolak
    #[schema(example = 20)]
    pub rejected: i64,
    
    /// Sisa kuota
    #[schema(example = 20)]
    pub remaining_quota: i64,
}
