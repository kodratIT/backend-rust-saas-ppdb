-- Create schools table
CREATE TABLE schools (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    npsn VARCHAR(20) UNIQUE NOT NULL,
    code VARCHAR(50) UNIQUE NOT NULL,
    address TEXT,
    phone VARCHAR(20),
    email VARCHAR(255),
    logo_url TEXT,
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'suspended')),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_schools_status ON schools(status);
CREATE INDEX idx_schools_code ON schools(code);
CREATE INDEX idx_schools_npsn ON schools(npsn);

-- Create trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_schools_updated_at BEFORE UPDATE ON schools
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
-- Create users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    school_id INTEGER REFERENCES schools(id) ON DELETE CASCADE,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    phone VARCHAR(20),
    nik VARCHAR(20),
    role VARCHAR(20) NOT NULL CHECK (role IN ('super_admin', 'school_admin', 'parent')),
    email_verified BOOLEAN DEFAULT FALSE,
    email_verification_token VARCHAR(255),
    reset_password_token VARCHAR(255),
    reset_password_expires TIMESTAMP,
    last_login_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_school_id ON users(school_id);
CREATE INDEX idx_users_role ON users(role);

-- Enable Row Level Security
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

-- Create RLS policy for tenant isolation
CREATE POLICY users_isolation ON users
    FOR ALL
    USING (
        role = 'super_admin' OR 
        school_id = current_setting('app.current_school_id', true)::int
    );

-- Create trigger for updated_at
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
-- Create periods table
CREATE TABLE periods (
    id SERIAL PRIMARY KEY,
    school_id INTEGER NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
    academic_year VARCHAR(20) NOT NULL,
    level VARCHAR(20) NOT NULL CHECK (level IN ('SD', 'SMP', 'SMA', 'SMK')),
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    registration_start DATE NOT NULL,
    registration_end DATE NOT NULL,
    announcement_date DATE,
    reenrollment_deadline DATE,
    status VARCHAR(20) DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'closed')),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT unique_period_per_school UNIQUE (school_id, academic_year, level)
);

-- Create indexes
CREATE INDEX idx_periods_school_id ON periods(school_id);
CREATE INDEX idx_periods_status ON periods(status);

-- Enable Row Level Security
ALTER TABLE periods ENABLE ROW LEVEL SECURITY;

-- Create RLS policy
CREATE POLICY periods_isolation ON periods
    FOR ALL
    USING (school_id = current_setting('app.current_school_id', true)::int);

-- Create trigger for updated_at
CREATE TRIGGER update_periods_updated_at BEFORE UPDATE ON periods
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
-- Create registration_paths table
CREATE TABLE registration_paths (
    id SERIAL PRIMARY KEY,
    period_id INTEGER NOT NULL REFERENCES periods(id) ON DELETE CASCADE,
    path_type VARCHAR(50) NOT NULL CHECK (path_type IN ('zonasi', 'prestasi', 'afirmasi', 'perpindahan_tugas')),
    quota INTEGER NOT NULL,
    description TEXT,
    scoring_config JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_paths_period_id ON registration_paths(period_id);

-- Create trigger for updated_at
CREATE TRIGGER update_registration_paths_updated_at BEFORE UPDATE ON registration_paths
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Example scoring_config JSONB structure:
-- {
--   "zonasi": {"max_distance_km": 5, "weight": 1.0},
--   "prestasi": {"rapor_weight": 0.6, "achievement_weight": 0.4}
-- }
-- Create registrations table
CREATE TABLE registrations (
    id SERIAL PRIMARY KEY,
    school_id INTEGER NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    period_id INTEGER NOT NULL REFERENCES periods(id) ON DELETE CASCADE,
    path_id INTEGER NOT NULL REFERENCES registration_paths(id),
    registration_number VARCHAR(50) UNIQUE NOT NULL,
    
    -- Student Data
    student_nisn VARCHAR(20) NOT NULL,
    student_nik VARCHAR(20) NOT NULL,
    student_name VARCHAR(255) NOT NULL,
    student_birth_place VARCHAR(100),
    student_birth_date DATE,
    student_gender VARCHAR(10) CHECK (student_gender IN ('L', 'P')),
    student_address TEXT,
    previous_school VARCHAR(255),
    
    -- Path-specific data (JSONB for flexibility)
    path_data JSONB,
    
    -- Selection
    selection_score DECIMAL(10, 2),
    ranking INTEGER,
    
    -- Status
    status VARCHAR(20) DEFAULT 'draft' CHECK (status IN (
        'draft', 'submitted', 'verified', 'rejected', 
        'accepted', 'enrolled', 'expired'
    )),
    rejection_reason TEXT,
    admin_notes TEXT,
    
    -- Timestamps
    submitted_at TIMESTAMP,
    verified_at TIMESTAMP,
    verified_by INTEGER REFERENCES users(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_registrations_school_id ON registrations(school_id);
CREATE INDEX idx_registrations_user_id ON registrations(user_id);
CREATE INDEX idx_registrations_period_id ON registrations(period_id);
CREATE INDEX idx_registrations_status ON registrations(status);
CREATE INDEX idx_registrations_number ON registrations(registration_number);
CREATE INDEX idx_registrations_ranking ON registrations(path_id, ranking);

-- Enable Row Level Security
ALTER TABLE registrations ENABLE ROW LEVEL SECURITY;

-- Create RLS policy
CREATE POLICY registrations_isolation ON registrations
    FOR ALL
    USING (school_id = current_setting('app.current_school_id', true)::int);

-- Create trigger for updated_at
CREATE TRIGGER update_registrations_updated_at BEFORE UPDATE ON registrations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
-- Create documents table
CREATE TABLE documents (
    id SERIAL PRIMARY KEY,
    registration_id INTEGER NOT NULL REFERENCES registrations(id) ON DELETE CASCADE,
    document_type VARCHAR(50) NOT NULL CHECK (document_type IN (
        'kartu_keluarga', 'akta_kelahiran', 'rapor', 
        'sertifikat_prestasi', 'surat_keterangan', 'other'
    )),
    file_url TEXT NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_size INTEGER,
    mime_type VARCHAR(100),
    verification_status VARCHAR(20) DEFAULT 'pending' CHECK (verification_status IN (
        'pending', 'approved', 'rejected'
    )),
    rejection_reason TEXT,
    verified_by INTEGER REFERENCES users(id),
    verified_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_documents_registration_id ON documents(registration_id);
CREATE INDEX idx_documents_verification_status ON documents(verification_status);

-- Create trigger for updated_at
CREATE TRIGGER update_documents_updated_at BEFORE UPDATE ON documents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
-- Create payments table
CREATE TABLE payments (
    id SERIAL PRIMARY KEY,
    registration_id INTEGER NOT NULL REFERENCES registrations(id) ON DELETE CASCADE,
    amount DECIMAL(15, 2) NOT NULL,
    payment_method VARCHAR(50) NOT NULL CHECK (payment_method IN (
        'bank_transfer', 'virtual_account', 'ewallet', 'qris'
    )),
    payment_code VARCHAR(100),
    payment_provider VARCHAR(50),
    
    -- Payment details (JSONB for flexibility)
    payment_details JSONB,
    
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN (
        'pending', 'paid', 'failed', 'expired', 'refunded'
    )),
    
    -- Proof for manual transfer
    proof_url TEXT,
    
    -- External reference
    external_id VARCHAR(255),
    external_status VARCHAR(50),
    
    paid_at TIMESTAMP,
    expired_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_payments_registration_id ON payments(registration_id);
CREATE INDEX idx_payments_status ON payments(status);
CREATE INDEX idx_payments_external_id ON payments(external_id);

-- Create trigger for updated_at
CREATE TRIGGER update_payments_updated_at BEFORE UPDATE ON payments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
-- Create audit_logs table
CREATE TABLE audit_logs (
    id SERIAL PRIMARY KEY,
    school_id INTEGER REFERENCES schools(id) ON DELETE CASCADE,
    user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id INTEGER NOT NULL,
    action VARCHAR(50) NOT NULL CHECK (action IN (
        'create', 'update', 'delete', 'login', 'logout', 
        'verify', 'reject', 'approve', 'payment'
    )),
    old_value JSONB,
    new_value JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_audit_logs_school_id ON audit_logs(school_id);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);

-- Enable Row Level Security
ALTER TABLE audit_logs ENABLE ROW LEVEL SECURITY;

-- Create RLS policy (read-only per tenant)
CREATE POLICY audit_logs_isolation ON audit_logs
    FOR SELECT
    USING (school_id = current_setting('app.current_school_id', true)::int);
-- Create federated_identities table
CREATE TABLE federated_identities (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    access_token TEXT,
    refresh_token TEXT,
    token_expires_at TIMESTAMP,
    sync_status VARCHAR(20) DEFAULT 'pending' CHECK (sync_status IN (
        'pending', 'synced', 'failed'
    )),
    last_sync_at TIMESTAMP,
    sync_error TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT unique_provider_user UNIQUE (provider, provider_user_id)
);

-- Create indexes
CREATE INDEX idx_federated_user_id ON federated_identities(user_id);
CREATE INDEX idx_federated_provider ON federated_identities(provider);

-- Create trigger for updated_at
CREATE TRIGGER update_federated_identities_updated_at BEFORE UPDATE ON federated_identities
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
-- Seed data for development

-- Insert sample schools
INSERT INTO schools (name, npsn, code, address, phone, email, status) VALUES
('SMA Negeri 1 Jakarta', '20100001', 'SMAN1JKT', 'Jl. Sudirman No. 1, Jakarta Pusat', '021-1234567', 'info@sman1jkt.sch.id', 'active'),
('SMP Negeri 5 Bandung', '20200005', 'SMPN5BDG', 'Jl. Asia Afrika No. 5, Bandung', '022-7654321', 'info@smpn5bdg.sch.id', 'active'),
('SD Negeri 10 Surabaya', '20300010', 'SDN10SBY', 'Jl. Pahlawan No. 10, Surabaya', '031-9876543', 'info@sdn10sby.sch.id', 'active');

-- Insert super admin (password: admin123)
-- Hash generated with argon2: $argon2id$v=19$m=19456,t=2,p=1$...
INSERT INTO users (school_id, email, password_hash, full_name, role, email_verified) VALUES
(NULL, 'superadmin@ppdb.com', '$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$qLml8cg7JQbGpQqP3vqZ0Q', 'Super Admin', 'super_admin', true);

-- Insert school admins for each school
INSERT INTO users (school_id, email, password_hash, full_name, phone, role, email_verified) VALUES
(1, 'admin@sman1jkt.sch.id', '$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$qLml8cg7JQbGpQqP3vqZ0Q', 'Admin SMA 1 Jakarta', '081234567890', 'school_admin', true),
(2, 'admin@smpn5bdg.sch.id', '$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$qLml8cg7JQbGpQqP3vqZ0Q', 'Admin SMP 5 Bandung', '082345678901', 'school_admin', true),
(3, 'admin@sdn10sby.sch.id', '$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$qLml8cg7JQbGpQqP3vqZ0Q', 'Admin SD 10 Surabaya', '083456789012', 'school_admin', true);

-- Insert sample parent users
INSERT INTO users (school_id, email, password_hash, full_name, phone, nik, role, email_verified) VALUES
(1, 'parent1@example.com', '$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$qLml8cg7JQbGpQqP3vqZ0Q', 'Budi Santoso', '081111111111', '3201010101010001', 'parent', true),
(2, 'parent2@example.com', '$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$qLml8cg7JQbGpQqP3vqZ0Q', 'Siti Rahayu', '082222222222', '3202020202020002', 'parent', true);

-- Insert sample periods
INSERT INTO periods (school_id, academic_year, level, start_date, end_date, registration_start, registration_end, announcement_date, reenrollment_deadline, status) VALUES
(1, '2025/2026', 'SMA', '2025-07-01', '2026-06-30', '2025-01-01', '2025-02-28', '2025-03-15', '2025-04-01', 'active'),
(2, '2025/2026', 'SMP', '2025-07-01', '2026-06-30', '2025-01-01', '2025-02-28', '2025-03-15', '2025-04-01', 'active'),
(3, '2025/2026', 'SD', '2025-07-01', '2026-06-30', '2025-01-01', '2025-02-28', '2025-03-15', '2025-04-01', 'active');

-- Insert registration paths for each period
INSERT INTO registration_paths (period_id, path_type, quota, description, scoring_config) VALUES
-- SMA Negeri 1 Jakarta
(1, 'zonasi', 100, 'Jalur Zonasi - berdasarkan jarak tempat tinggal', '{"max_distance_km": 5, "weight": 1.0}'),
(1, 'prestasi', 50, 'Jalur Prestasi - berdasarkan nilai rapor dan prestasi', '{"rapor_weight": 0.6, "achievement_weight": 0.4}'),
(1, 'afirmasi', 20, 'Jalur Afirmasi - untuk siswa kurang mampu', '{"criteria": "economic_status"}'),
(1, 'perpindahan_tugas', 10, 'Jalur Perpindahan Tugas Orang Tua', '{"document_required": "surat_tugas"}'),

-- SMP Negeri 5 Bandung
(2, 'zonasi', 80, 'Jalur Zonasi - berdasarkan jarak tempat tinggal', '{"max_distance_km": 3, "weight": 1.0}'),
(2, 'prestasi', 40, 'Jalur Prestasi - berdasarkan nilai rapor dan prestasi', '{"rapor_weight": 0.7, "achievement_weight": 0.3}'),
(2, 'afirmasi', 15, 'Jalur Afirmasi - untuk siswa kurang mampu', '{"criteria": "economic_status"}'),

-- SD Negeri 10 Surabaya
(3, 'zonasi', 60, 'Jalur Zonasi - berdasarkan jarak tempat tinggal', '{"max_distance_km": 2, "weight": 1.0}'),
(3, 'prestasi', 20, 'Jalur Prestasi - berdasarkan prestasi TK', '{"tk_report_weight": 1.0}'),
(3, 'afirmasi', 10, 'Jalur Afirmasi - untuk siswa kurang mampu', '{"criteria": "economic_status"}');
