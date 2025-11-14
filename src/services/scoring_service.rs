use serde_json::Value;

use crate::models::registration::Registration;
use crate::utils::error::{AppError, AppResult};

pub struct ScoringService;

impl ScoringService {
    pub fn new() -> Self {
        Self
    }

    /// Calculate score for Zonasi path (based on distance)
    /// Closer distance = higher score
    /// Formula: 100 - (distance_km * weight)
    pub fn calculate_zonasi_score(
        &self,
        registration: &Registration,
        scoring_config: &Value,
    ) -> AppResult<f64> {
        // Extract distance from path_data
        let distance = registration
            .path_data
            .get("distance_km")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| {
                AppError::Validation("Distance (distance_km) is required for Zonasi path".to_string())
            })?;

        // Get weight from scoring_config (default: 2.0)
        let weight = scoring_config
            .get("distance_weight")
            .and_then(|v| v.as_f64())
            .unwrap_or(2.0);

        // Calculate score: closer = higher score
        let score = (100.0 - (distance * weight)).max(0.0);

        Ok(score)
    }

    /// Calculate score for Prestasi path (based on academic achievement)
    /// Formula: (rapor_average * rapor_weight) + (achievement_points * achievement_weight)
    pub fn calculate_prestasi_score(
        &self,
        registration: &Registration,
        scoring_config: &Value,
    ) -> AppResult<f64> {
        // Extract rapor average from path_data
        let rapor_average = registration
            .path_data
            .get("rapor_average")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| {
                AppError::Validation("Rapor average is required for Prestasi path".to_string())
            })?;

        // Validate rapor average (0-100)
        if rapor_average < 0.0 || rapor_average > 100.0 {
            return Err(AppError::Validation(
                "Rapor average must be between 0 and 100".to_string(),
            ));
        }

        // Extract achievement points (optional)
        let achievement_points = registration
            .path_data
            .get("achievement_points")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Get weights from scoring_config
        let rapor_weight = scoring_config
            .get("rapor_weight")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.7);

        let achievement_weight = scoring_config
            .get("achievement_weight")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.3);

        // Calculate score
        let score = (rapor_average * rapor_weight) + (achievement_points * achievement_weight);

        Ok(score.min(100.0))
    }

    /// Calculate score for Afirmasi path (based on criteria)
    /// Formula: base_score + bonus points for meeting criteria
    pub fn calculate_afirmasi_score(
        &self,
        registration: &Registration,
        scoring_config: &Value,
    ) -> AppResult<f64> {
        let mut score = 50.0; // Base score

        // Check if has KIP (Kartu Indonesia Pintar)
        if let Some(has_kip) = registration.path_data.get("has_kip").and_then(|v| v.as_bool()) {
            if has_kip {
                let kip_bonus = scoring_config
                    .get("kip_bonus")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(20.0);
                score += kip_bonus;
            }
        }

        // Check if from poor family
        if let Some(is_poor) = registration.path_data.get("is_poor_family").and_then(|v| v.as_bool()) {
            if is_poor {
                let poor_bonus = scoring_config
                    .get("poor_family_bonus")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(15.0);
                score += poor_bonus;
            }
        }

        // Check if has disability
        if let Some(has_disability) = registration.path_data.get("has_disability").and_then(|v| v.as_bool()) {
            if has_disability {
                let disability_bonus = scoring_config
                    .get("disability_bonus")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(15.0);
                score += disability_bonus;
            }
        }

        // Add rapor score if available
        if let Some(rapor_average) = registration.path_data.get("rapor_average").and_then(|v| v.as_f64()) {
            let rapor_weight = scoring_config
                .get("rapor_weight")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.3);
            score += rapor_average * rapor_weight;
        }

        Ok(score.min(100.0))
    }

    /// Calculate score for Perpindahan Tugas path
    /// Formula: base_score + document completeness
    pub fn calculate_perpindahan_score(
        &self,
        registration: &Registration,
        scoring_config: &Value,
    ) -> AppResult<f64> {
        let mut score = 60.0; // Base score

        // Check if has valid transfer letter
        if let Some(has_letter) = registration
            .path_data
            .get("has_transfer_letter")
            .and_then(|v| v.as_bool())
        {
            if has_letter {
                let letter_bonus = scoring_config
                    .get("transfer_letter_bonus")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(20.0);
                score += letter_bonus;
            }
        }

        // Check if has parent assignment letter
        if let Some(has_assignment) = registration
            .path_data
            .get("has_parent_assignment")
            .and_then(|v| v.as_bool())
        {
            if has_assignment {
                let assignment_bonus = scoring_config
                    .get("parent_assignment_bonus")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(20.0);
                score += assignment_bonus;
            }
        }

        Ok(score.min(100.0))
    }

    /// Calculate score based on path type
    pub fn calculate_score(
        &self,
        registration: &Registration,
        path_type: &str,
        scoring_config: &Value,
    ) -> AppResult<f64> {
        match path_type {
            "zonasi" => self.calculate_zonasi_score(registration, scoring_config),
            "prestasi" => self.calculate_prestasi_score(registration, scoring_config),
            "afirmasi" => self.calculate_afirmasi_score(registration, scoring_config),
            "perpindahan_tugas" => self.calculate_perpindahan_score(registration, scoring_config),
            _ => Err(AppError::Validation(format!(
                "Unknown path type: {}",
                path_type
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_calculate_zonasi_score() {
        let service = ScoringService::new();
        
        let mut registration = Registration {
            id: 1,
            school_id: 1,
            user_id: 1,
            period_id: 1,
            path_id: 1,
            registration_number: None,
            student_nisn: "1234567890".to_string(),
            student_name: "Test Student".to_string(),
            student_gender: "L".to_string(),
            student_birth_place: "Jakarta".to_string(),
            student_birth_date: chrono::Utc::now(),
            student_religion: "Islam".to_string(),
            student_address: "Test Address".to_string(),
            student_phone: None,
            student_email: None,
            parent_name: "Test Parent".to_string(),
            parent_nik: "1234567890123456".to_string(),
            parent_phone: "081234567890".to_string(),
            parent_occupation: None,
            parent_income: None,
            previous_school_name: None,
            previous_school_npsn: None,
            previous_school_address: None,
            path_data: json!({"distance_km": 2.5}),
            selection_score: None,
            ranking: None,
            status: "verified".to_string(),
            rejection_reason: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let scoring_config = json!({"distance_weight": 2.0});
        
        let score = service.calculate_zonasi_score(&registration, &scoring_config).unwrap();
        assert_eq!(score, 95.0); // 100 - (2.5 * 2.0) = 95.0
    }

    #[test]
    fn test_calculate_prestasi_score() {
        let service = ScoringService::new();
        
        let registration = Registration {
            path_data: json!({
                "rapor_average": 85.0,
                "achievement_points": 10.0
            }),
            // ... other fields
            ..Default::default()
        };

        let scoring_config = json!({
            "rapor_weight": 0.7,
            "achievement_weight": 0.3
        });
        
        let score = service.calculate_prestasi_score(&registration, &scoring_config).unwrap();
        // (85 * 0.7) + (10 * 0.3) = 59.5 + 3.0 = 62.5
        assert_eq!(score, 62.5);
    }
}
