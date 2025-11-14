use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Period {
    pub id: i32,
    pub school_id: i32,
    pub academic_year: String,
    pub level: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub announcement_date: Option<DateTime<Utc>>,
    pub reenrollment_deadline: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeriodStatus {
    Draft,
    Active,
    Closed,
}

impl PeriodStatus {
    pub fn as_str(&self) -> &str {
        match self {
            PeriodStatus::Draft => "draft",
            PeriodStatus::Active => "active",
            PeriodStatus::Closed => "closed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(PeriodStatus::Draft),
            "active" => Some(PeriodStatus::Active),
            "closed" => Some(PeriodStatus::Closed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Level {
    SD,
    SMP,
    SMA,
    SMK,
}

impl Level {
    pub fn as_str(&self) -> &str {
        match self {
            Level::SD => "SD",
            Level::SMP => "SMP",
            Level::SMA => "SMA",
            Level::SMK => "SMK",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "SD" => Some(Level::SD),
            "SMP" => Some(Level::SMP),
            "SMA" => Some(Level::SMA),
            "SMK" => Some(Level::SMK),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RegistrationPath {
    pub id: i32,
    pub period_id: i32,
    pub path_type: String,
    pub name: String,
    pub quota: i32,
    pub description: Option<String>,
    pub scoring_config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathType {
    Zonasi,
    Prestasi,
    Afirmasi,
    PerpindahanTugas,
}

impl PathType {
    pub fn as_str(&self) -> &str {
        match self {
            PathType::Zonasi => "zonasi",
            PathType::Prestasi => "prestasi",
            PathType::Afirmasi => "afirmasi",
            PathType::PerpindahanTugas => "perpindahan_tugas",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "zonasi" => Some(PathType::Zonasi),
            "prestasi" => Some(PathType::Prestasi),
            "afirmasi" => Some(PathType::Afirmasi),
            "perpindahan_tugas" => Some(PathType::PerpindahanTugas),
            _ => None,
        }
    }
}
