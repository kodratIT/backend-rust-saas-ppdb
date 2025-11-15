use crate::models::registration::Registration;
use crate::repositories::period_repo::PeriodRepository;
use crate::repositories::registration_repo::RegistrationRepository;
use crate::services::scoring_service::ScoringService;
use crate::utils::error::{AppError, AppResult};

pub struct SelectionService {
    registration_repo: RegistrationRepository,
    period_repo: PeriodRepository,
    scoring_service: ScoringService,
}

impl SelectionService {
    pub fn new(
        registration_repo: RegistrationRepository,
        period_repo: PeriodRepository,
    ) -> Self {
        Self {
            registration_repo,
            period_repo,
            scoring_service: ScoringService::new(),
        }
    }

    /// Calculate scores for all verified registrations in a period
    pub async fn calculate_scores_for_period(&self, period_id: i32) -> AppResult<usize> {
        // Check if period exists
        let period = self
            .period_repo
            .find_by_id(period_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Period not found".to_string()))?;

        // Get all paths for this period
        let paths = self.period_repo.find_paths_by_period(period_id).await?;

        let mut total_calculated = 0;

        // Calculate scores for each path
        for path in paths {
            // Get all verified registrations for this path
            let registrations = self
                .registration_repo
                .find_by_school(
                    period.school_id,
                    1000, // Large limit to get all
                    0,
                    Some("verified".to_string()),
                    Some(period_id),
                    Some(path.id),
                )
                .await?;

            // Calculate score for each registration
            for registration in registrations {
                let score = self
                    .scoring_service
                    .calculate_score(&registration, &path.path_type, &path.scoring_config)?;

                // Update registration with calculated score
                sqlx::query(
                    r#"
                    UPDATE registrations 
                    SET selection_score = $2, updated_at = NOW()
                    WHERE id = $1
                    "#,
                )
                .bind(registration.id)
                .bind(score)
                .execute(&self.registration_repo.pool)
                .await?;

                total_calculated += 1;
            }
        }

        tracing::info!(
            "Calculated scores for {} registrations in period {}",
            total_calculated,
            period_id
        );

        Ok(total_calculated)
    }

    /// Update rankings for all registrations in a period
    /// Rankings are calculated per path, ordered by selection_score DESC
    pub async fn update_rankings(&self, period_id: i32) -> AppResult<usize> {
        // Check if period exists
        let period = self
            .period_repo
            .find_by_id(period_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Period not found".to_string()))?;

        // Get all paths for this period
        let paths = self.period_repo.find_paths_by_period(period_id).await?;

        let mut total_ranked = 0;

        // Update rankings for each path
        for path in paths {
            // Get all verified registrations with scores for this path, ordered by score DESC
            let registrations = sqlx::query_as::<_, Registration>(
                r#"
                SELECT * FROM registrations 
                WHERE school_id = $1 
                  AND period_id = $2 
                  AND path_id = $3 
                  AND status = 'verified'
                  AND selection_score IS NOT NULL
                ORDER BY selection_score DESC, created_at ASC
                "#,
            )
            .bind(period.school_id)
            .bind(period_id)
            .bind(path.id)
            .fetch_all(&self.registration_repo.pool)
            .await?;

            // Update ranking for each registration
            for (index, registration) in registrations.iter().enumerate() {
                let ranking = (index + 1) as i32;

                sqlx::query(
                    r#"
                    UPDATE registrations 
                    SET ranking = $2, updated_at = NOW()
                    WHERE id = $1
                    "#,
                )
                .bind(registration.id)
                .bind(ranking)
                .execute(&self.registration_repo.pool)
                .await?;

                total_ranked += 1;
            }
        }

        tracing::info!(
            "Updated rankings for {} registrations in period {}",
            total_ranked,
            period_id
        );

        Ok(total_ranked)
    }

    /// Get rankings for a specific path
    pub async fn get_rankings(
        &self,
        period_id: i32,
        path_id: i32,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<Registration>> {
        let registrations = sqlx::query_as::<_, Registration>(
            r#"
            SELECT * FROM registrations 
            WHERE period_id = $1 
              AND path_id = $2 
              AND status = 'verified'
              AND selection_score IS NOT NULL
              AND ranking IS NOT NULL
            ORDER BY ranking ASC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(period_id)
        .bind(path_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.registration_repo.pool)
        .await?;

        Ok(registrations)
    }

    /// Get ranking statistics for a period
    pub async fn get_ranking_statistics(&self, period_id: i32) -> AppResult<Vec<PathRankingStats>> {
        // Get all paths for this period
        let paths = self.period_repo.find_paths_by_period(period_id).await?;

        let mut stats = Vec::new();

        for path in paths {
            // Count registrations with scores
            let total_with_score: i64 = sqlx::query_scalar(
                r#"
                SELECT COUNT(*) FROM registrations 
                WHERE period_id = $1 
                  AND path_id = $2 
                  AND status = 'verified'
                  AND selection_score IS NOT NULL
                "#,
            )
            .bind(period_id)
            .bind(path.id)
            .fetch_one(&self.registration_repo.pool)
            .await?;

            // Get highest score
            let highest_score: Option<f64> = sqlx::query_scalar(
                r#"
                SELECT MAX(selection_score) FROM registrations 
                WHERE period_id = $1 
                  AND path_id = $2 
                  AND status = 'verified'
                "#,
            )
            .bind(period_id)
            .bind(path.id)
            .fetch_one(&self.registration_repo.pool)
            .await?;

            // Get lowest score
            let lowest_score: Option<f64> = sqlx::query_scalar(
                r#"
                SELECT MIN(selection_score) FROM registrations 
                WHERE period_id = $1 
                  AND path_id = $2 
                  AND status = 'verified'
                  AND selection_score IS NOT NULL
                "#,
            )
            .bind(period_id)
            .bind(path.id)
            .fetch_one(&self.registration_repo.pool)
            .await?;

            // Get average score
            let average_score: Option<f64> = sqlx::query_scalar(
                r#"
                SELECT AVG(selection_score) FROM registrations 
                WHERE period_id = $1 
                  AND path_id = $2 
                  AND status = 'verified'
                  AND selection_score IS NOT NULL
                "#,
            )
            .bind(period_id)
            .bind(path.id)
            .fetch_one(&self.registration_repo.pool)
            .await?;

            stats.push(PathRankingStats {
                path_id: path.id,
                path_name: path.name,
                path_type: path.path_type,
                quota: path.quota,
                total_registrations: total_with_score,
                highest_score,
                lowest_score,
                average_score,
            });
        }

        Ok(stats)
    }
}

/// Statistik ranking per jalur pendaftaran
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PathRankingStats {
    /// ID jalur
    #[schema(example = 1)]
    pub path_id: i32,
    
    /// Nama jalur
    #[schema(example = "Jalur Zonasi")]
    pub path_name: String,
    
    /// Tipe jalur
    #[schema(example = "zonasi")]
    pub path_type: String,
    
    /// Kuota jalur
    #[schema(example = 100)]
    pub quota: i32,
    
    /// Total pendaftaran
    #[schema(example = 150)]
    pub total_registrations: i64,
    
    /// Skor tertinggi
    #[schema(example = 95.5)]
    pub highest_score: Option<f64>,
    
    /// Skor terendah
    #[schema(example = 65.0)]
    pub lowest_score: Option<f64>,
    
    /// Skor rata-rata
    #[schema(example = 80.5)]
    pub average_score: Option<f64>,
}
