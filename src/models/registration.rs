use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct Registration {
    pub id: i32,
    pub school_id: i32,
    pub user_id: i32,
    pub period_id: i32,
    pub path_id: i32,
    pub registration_number: Option<String>,
    
    // Student data
    pub student_nisn: String,
    pub student_name: String,
    pub student_gender: String,
    pub student_birth_place: String,
    pub student_birth_date: DateTime<Utc>,
    pub student_religion: String,
    pub student_address: String,
    pub student_phone: Option<String>,
    pub student_email: Option<String>,
    
    // Parent data
    pub parent_name: String,
    pub parent_nik: String,
    pub parent_phone: String,
    pub parent_occupation: Option<String>,
    pub parent_income: Option<String>,
    
    // Previous school data
    pub previous_school_name: Option<String>,
    pub previous_school_npsn: Option<String>,
    pub previous_school_address: Option<String>,
    
    // Path-specific data (JSONB)
    pub path_data: serde_json::Value,
    
    // Selection data
    pub selection_score: Option<f64>,
    pub ranking: Option<i32>,
    
    // Status
    pub status: String,
    pub rejection_reason: Option<String>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistrationStatus {
    Draft,
    Submitted,
    Verified,
    Rejected,
    Accepted,
    Enrolled,
    Expired,
}

impl RegistrationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            RegistrationStatus::Draft => "draft",
            RegistrationStatus::Submitted => "submitted",
            RegistrationStatus::Verified => "verified",
            RegistrationStatus::Rejected => "rejected",
            RegistrationStatus::Accepted => "accepted",
            RegistrationStatus::Enrolled => "enrolled",
            RegistrationStatus::Expired => "expired",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(RegistrationStatus::Draft),
            "submitted" => Some(RegistrationStatus::Submitted),
            "verified" => Some(RegistrationStatus::Verified),
            "rejected" => Some(RegistrationStatus::Rejected),
            "accepted" => Some(RegistrationStatus::Accepted),
            "enrolled" => Some(RegistrationStatus::Enrolled),
            "expired" => Some(RegistrationStatus::Expired),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: i32,
    pub registration_id: i32,
    pub document_type: String,
    pub file_url: String,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub verification_status: String,
    pub verification_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    KartuKeluarga,
    AktaKelahiran,
    Ijazah,
    Rapor,
    SertifikatPrestasi,
    SuratKeteranganPindah,
    SuratKeteranganAfirmasi,
    Foto,
    Other,
}

impl DocumentType {
    pub fn as_str(&self) -> &str {
        match self {
            DocumentType::KartuKeluarga => "kartu_keluarga",
            DocumentType::AktaKelahiran => "akta_kelahiran",
            DocumentType::Ijazah => "ijazah",
            DocumentType::Rapor => "rapor",
            DocumentType::SertifikatPrestasi => "sertifikat_prestasi",
            DocumentType::SuratKeteranganPindah => "surat_keterangan_pindah",
            DocumentType::SuratKeteranganAfirmasi => "surat_keterangan_afirmasi",
            DocumentType::Foto => "foto",
            DocumentType::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "kartu_keluarga" => Some(DocumentType::KartuKeluarga),
            "akta_kelahiran" => Some(DocumentType::AktaKelahiran),
            "ijazah" => Some(DocumentType::Ijazah),
            "rapor" => Some(DocumentType::Rapor),
            "sertifikat_prestasi" => Some(DocumentType::SertifikatPrestasi),
            "surat_keterangan_pindah" => Some(DocumentType::SuratKeteranganPindah),
            "surat_keterangan_afirmasi" => Some(DocumentType::SuratKeteranganAfirmasi),
            "foto" => Some(DocumentType::Foto),
            "other" => Some(DocumentType::Other),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Pending,
    Approved,
    Rejected,
}

impl VerificationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            VerificationStatus::Pending => "pending",
            VerificationStatus::Approved => "approved",
            VerificationStatus::Rejected => "rejected",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(VerificationStatus::Pending),
            "approved" => Some(VerificationStatus::Approved),
            "rejected" => Some(VerificationStatus::Rejected),
            _ => None,
        }
    }
}
