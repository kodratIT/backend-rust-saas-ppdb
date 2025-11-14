use crate::models::school::School;
use crate::repositories::school_repo::SchoolRepository;
use crate::utils::error::{AppError, AppResult};

pub struct SchoolService {
    school_repo: SchoolRepository,
}

impl SchoolService {
    pub fn new(school_repo: SchoolRepository) -> Self {
        Self { school_repo }
    }

    pub async fn create_school(
        &self,
        name: String,
        npsn: String,
        code: String,
        address: Option<String>,
        phone: Option<String>,
        email: Option<String>,
        logo_url: Option<String>,
    ) -> AppResult<School> {
        // Check if NPSN already exists
        if let Some(_) = self.school_repo.find_by_npsn(&npsn).await? {
            return Err(AppError::Conflict("NPSN already registered".to_string()));
        }

        // Check if code already exists
        if let Some(_) = self.school_repo.find_by_code(&code).await? {
            return Err(AppError::Conflict("School code already exists".to_string()));
        }

        // Create school
        let school = self
            .school_repo
            .create_school(name, npsn, code, address, phone, email, logo_url)
            .await?;

        Ok(school)
    }

    pub async fn list_schools(
        &self,
        page: i64,
        page_size: i64,
        search: Option<String>,
        status: Option<String>,
    ) -> AppResult<(Vec<School>, i64)> {
        let offset = (page - 1) * page_size;
        
        let schools = self
            .school_repo
            .find_all(page_size, offset, search.clone(), status.clone())
            .await?;

        let total = self
            .school_repo
            .count_all(search, status)
            .await?;

        Ok((schools, total))
    }

    pub async fn get_school(&self, id: i32) -> AppResult<School> {
        self.school_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("School not found".to_string()))
    }

    pub async fn update_school(
        &self,
        id: i32,
        name: Option<String>,
        address: Option<String>,
        phone: Option<String>,
        email: Option<String>,
        logo_url: Option<String>,
    ) -> AppResult<School> {
        // Check if school exists
        let _school = self.get_school(id).await?;

        // Update school
        let updated_school = self
            .school_repo
            .update_school(id, name, address, phone, email, logo_url)
            .await?;

        Ok(updated_school)
    }

    pub async fn deactivate_school(&self, id: i32) -> AppResult<()> {
        // Check if school exists
        let _school = self.get_school(id).await?;

        // Deactivate school (soft delete)
        self.school_repo.update_status(id, "inactive").await?;

        Ok(())
    }

    pub async fn activate_school(&self, id: i32) -> AppResult<()> {
        // Check if school exists
        let _school = self.get_school(id).await?;

        // Activate school
        self.school_repo.update_status(id, "active").await?;

        Ok(())
    }
}
