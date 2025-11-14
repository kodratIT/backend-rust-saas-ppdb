use chrono::{DateTime, Utc};

use crate::models::period::{Period, RegistrationPath};
use crate::repositories::period_repo::PeriodRepository;
use crate::utils::error::{AppError, AppResult};

pub struct PeriodService {
    period_repo: PeriodRepository,
}

impl PeriodService {
    pub fn new(period_repo: PeriodRepository) -> Self {
        Self { period_repo }
    }

    pub async fn create_period(
        &self,
        school_id: i32,
        academic_year: String,
        level: String,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        reenrollment_deadline: Option<DateTime<Utc>>,
    ) -> AppResult<Period> {
        // Validate dates
        if end_date <= start_date {
            return Err(AppError::Validation(
                "End date must be after start date".to_string(),
            ));
        }

        if let Some(deadline) = reenrollment_deadline {
            if deadline <= end_date {
                return Err(AppError::Validation(
                    "Re-enrollment deadline must be after end date".to_string(),
                ));
            }
        }

        // Check if there's already an active period for this school/year/level
        if let Some(_) = self
            .period_repo
            .find_active_by_school_and_level(school_id, &academic_year, &level)
            .await?
        {
            return Err(AppError::Conflict(
                "An active period already exists for this academic year and level".to_string(),
            ));
        }

        // Create period
        let period = self
            .period_repo
            .create_period(
                school_id,
                &academic_year,
                &level,
                start_date,
                end_date,
                reenrollment_deadline,
            )
            .await?;

        Ok(period)
    }

    pub async fn list_periods(
        &self,
        school_id: i32,
        page: i64,
        page_size: i64,
        status: Option<String>,
        academic_year: Option<String>,
        level: Option<String>,
    ) -> AppResult<(Vec<Period>, i64)> {
        let offset = (page - 1) * page_size;

        let periods = self
            .period_repo
            .find_by_school(school_id, page_size, offset, status.clone(), academic_year.clone(), level.clone())
            .await?;

        let total = self
            .period_repo
            .count_by_school(school_id, status, academic_year, level)
            .await?;

        Ok((periods, total))
    }

    pub async fn get_period(&self, id: i32) -> AppResult<Period> {
        self.period_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Period not found".to_string()))
    }

    pub async fn get_period_with_paths(&self, id: i32) -> AppResult<(Period, Vec<RegistrationPath>)> {
        let period = self.get_period(id).await?;
        let paths = self.period_repo.find_paths_by_period(id).await?;

        Ok((period, paths))
    }

    pub async fn update_period(
        &self,
        id: i32,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        announcement_date: Option<DateTime<Utc>>,
        reenrollment_deadline: Option<DateTime<Utc>>,
    ) -> AppResult<Period> {
        // Check if period exists
        let period = self.get_period(id).await?;

        // Only allow updates if status is draft
        if period.status != "draft" {
            return Err(AppError::Validation(
                "Can only update periods in draft status".to_string(),
            ));
        }

        // Validate dates if provided
        if let (Some(start), Some(end)) = (start_date, end_date) {
            if end <= start {
                return Err(AppError::Validation(
                    "End date must be after start date".to_string(),
                ));
            }
        }

        // Update period
        let updated_period = self
            .period_repo
            .update_period(id, start_date, end_date, announcement_date, reenrollment_deadline)
            .await?;

        Ok(updated_period)
    }

    pub async fn activate_period(&self, id: i32) -> AppResult<Period> {
        let period = self.get_period(id).await?;

        // Check if already active
        if period.status == "active" {
            return Err(AppError::Validation("Period is already active".to_string()));
        }

        // Check if there's already an active period for this school/year/level
        if let Some(active_period) = self
            .period_repo
            .find_active_by_school_and_level(period.school_id, &period.academic_year, &period.level)
            .await?
        {
            // Deactivate the existing active period
            self.period_repo
                .update_status(active_period.id, "closed")
                .await?;
        }

        // Activate this period
        let activated_period = self.period_repo.update_status(id, "active").await?;

        Ok(activated_period)
    }

    pub async fn close_period(&self, id: i32) -> AppResult<Period> {
        let period = self.get_period(id).await?;

        // Check if already closed
        if period.status == "closed" {
            return Err(AppError::Validation("Period is already closed".to_string()));
        }

        // Close period
        let closed_period = self.period_repo.update_status(id, "closed").await?;

        Ok(closed_period)
    }

    pub async fn delete_period(&self, id: i32) -> AppResult<()> {
        let period = self.get_period(id).await?;

        // Only allow deletion if status is draft
        if period.status != "draft" {
            return Err(AppError::Validation(
                "Can only delete periods in draft status".to_string(),
            ));
        }

        // Delete period (will cascade delete paths)
        self.period_repo.delete_period(id).await?;

        Ok(())
    }

    // Registration Path methods
    pub async fn create_path(
        &self,
        period_id: i32,
        path_type: String,
        name: String,
        quota: i32,
        description: Option<String>,
        scoring_config: serde_json::Value,
    ) -> AppResult<RegistrationPath> {
        // Check if period exists
        let period = self.get_period(period_id).await?;

        // Only allow adding paths if period is draft
        if period.status != "draft" {
            return Err(AppError::Validation(
                "Can only add paths to periods in draft status".to_string(),
            ));
        }

        // Validate quota
        if quota <= 0 {
            return Err(AppError::Validation("Quota must be greater than 0".to_string()));
        }

        // Create path
        let path = self
            .period_repo
            .create_path(
                period_id,
                &path_type,
                &name,
                quota,
                description.as_deref(),
                scoring_config,
            )
            .await?;

        Ok(path)
    }

    pub async fn get_paths_by_period(&self, period_id: i32) -> AppResult<Vec<RegistrationPath>> {
        // Check if period exists
        let _ = self.get_period(period_id).await?;

        let paths = self.period_repo.find_paths_by_period(period_id).await?;

        Ok(paths)
    }

    pub async fn get_path(&self, id: i32) -> AppResult<RegistrationPath> {
        self.period_repo
            .find_path_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Registration path not found".to_string()))
    }

    pub async fn update_path(
        &self,
        id: i32,
        name: Option<String>,
        quota: Option<i32>,
        description: Option<String>,
        scoring_config: Option<serde_json::Value>,
    ) -> AppResult<RegistrationPath> {
        // Check if path exists
        let path = self.get_path(id).await?;

        // Check if period is in draft status
        let period = self.get_period(path.period_id).await?;
        if period.status != "draft" {
            return Err(AppError::Validation(
                "Can only update paths for periods in draft status".to_string(),
            ));
        }

        // Validate quota if provided
        if let Some(q) = quota {
            if q <= 0 {
                return Err(AppError::Validation("Quota must be greater than 0".to_string()));
            }
        }

        // Update path
        let updated_path = self
            .period_repo
            .update_path(
                id,
                name.as_deref(),
                quota,
                description.as_deref(),
                scoring_config,
            )
            .await?;

        Ok(updated_path)
    }

    pub async fn delete_path(&self, id: i32) -> AppResult<()> {
        // Check if path exists
        let path = self.get_path(id).await?;

        // Check if period is in draft status
        let period = self.get_period(path.period_id).await?;
        if period.status != "draft" {
            return Err(AppError::Validation(
                "Can only delete paths for periods in draft status".to_string(),
            ));
        }

        // Delete path
        self.period_repo.delete_path(id).await?;

        Ok(())
    }
}
