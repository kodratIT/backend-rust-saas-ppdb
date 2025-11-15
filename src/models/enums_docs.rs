//! OpenAPI Documentation for Common Enums
//! 
//! This module provides ToSchema implementations for all enums used in the API.
//! Import these in your API handlers to include them in OpenAPI documentation.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// User role in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "school_admin")]
pub enum UserRole {
    /// Super administrator with access to all schools
    #[serde(rename = "super_admin")]
    SuperAdmin,
    
    /// School administrator with access to own school only
    #[serde(rename = "school_admin")]
    SchoolAdmin,
    
    /// Parent/guardian with access to own registrations only
    #[serde(rename = "parent")]
    Parent,
}

/// School status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "active")]
pub enum SchoolStatus {
    /// School is active and accepting registrations
    #[serde(rename = "active")]
    Active,
    
    /// School is inactive (soft deleted)
    #[serde(rename = "inactive")]
    Inactive,
}

/// PPDB period status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "active")]
pub enum PeriodStatus {
    /// Period is in draft mode (not yet active)
    #[serde(rename = "draft")]
    Draft,
    
    /// Period is active and accepting registrations
    #[serde(rename = "active")]
    Active,
    
    /// Period is closed (no longer accepting registrations)
    #[serde(rename = "closed")]
    Closed,
}

/// Education level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
#[schema(example = "SMA")]
pub enum Level {
    /// Sekolah Dasar (Elementary School)
    SD,
    
    /// Sekolah Menengah Pertama (Junior High School)
    SMP,
    
    /// Sekolah Menengah Atas (Senior High School)
    SMA,
    
    /// Sekolah Menengah Kejuruan (Vocational High School)
    SMK,
}

/// Registration path type (jalur pendaftaran)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "zonasi")]
pub enum PathType {
    /// Zonasi (proximity-based) - based on distance from school
    #[serde(rename = "zonasi")]
    Zonasi,
    
    /// Prestasi (achievement-based) - based on academic/non-academic achievements
    #[serde(rename = "prestasi")]
    Prestasi,
    
    /// Afirmasi (affirmative action) - for underprivileged students
    #[serde(rename = "afirmasi")]
    Afirmasi,
    
    /// Perpindahan Tugas Orang Tua (parent job transfer)
    #[serde(rename = "perpindahan_tugas")]
    PerpindahanTugas,
}

/// Student registration status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "submitted")]
pub enum RegistrationStatus {
    /// Registration is in draft mode (not yet submitted)
    #[serde(rename = "draft")]
    Draft,
    
    /// Registration has been submitted and awaiting verification
    #[serde(rename = "submitted")]
    Submitted,
    
    /// Registration has been verified by admin
    #[serde(rename = "verified")]
    Verified,
    
    /// Registration has been rejected
    #[serde(rename = "rejected")]
    Rejected,
    
    /// Student has been accepted (passed selection)
    #[serde(rename = "accepted")]
    Accepted,
    
    /// Student has completed re-enrollment
    #[serde(rename = "enrolled")]
    Enrolled,
    
    /// Registration expired (didn't complete re-enrollment)
    #[serde(rename = "expired")]
    Expired,
}

/// Document type for registration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "kartu_keluarga")]
pub enum DocumentType {
    /// Kartu Keluarga (Family Card)
    #[serde(rename = "kartu_keluarga")]
    KartuKeluarga,
    
    /// Akta Kelahiran (Birth Certificate)
    #[serde(rename = "akta_kelahiran")]
    AktaKelahiran,
    
    /// Rapor (Report Card)
    #[serde(rename = "rapor")]
    Rapor,
    
    /// Sertifikat Prestasi (Achievement Certificate)
    #[serde(rename = "sertifikat_prestasi")]
    SertifikatPrestasi,
    
    /// Surat Keterangan Tidak Mampu (Certificate of Financial Hardship)
    #[serde(rename = "sktm")]
    SKTM,
    
    /// Surat Tugas Orang Tua (Parent Job Transfer Letter)
    #[serde(rename = "surat_tugas")]
    SuratTugas,
    
    /// Other documents
    #[serde(rename = "lainnya")]
    Lainnya,
}

/// Document verification status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "pending")]
pub enum VerificationStatus {
    /// Document is pending verification
    #[serde(rename = "pending")]
    Pending,
    
    /// Document has been approved
    #[serde(rename = "approved")]
    Approved,
    
    /// Document has been rejected
    #[serde(rename = "rejected")]
    Rejected,
}
