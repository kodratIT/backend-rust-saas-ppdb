-- Create registrations table
CREATE TABLE registrations (
    id SERIAL PRIMARY KEY,
    school_id INTEGER NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    period_id INTEGER NOT NULL REFERENCES periods(id) ON DELETE CASCADE,
    path_id INTEGER NOT NULL REFERENCES registration_paths(id),
    registration_number VARCHAR(50) UNIQUE,
    
    -- Student Data
    student_nisn VARCHAR(20) NOT NULL,
    student_name VARCHAR(255) NOT NULL,
    student_gender VARCHAR(10) CHECK (student_gender IN ('L', 'P')),
    student_birth_place VARCHAR(100),
    student_birth_date TIMESTAMPTZ,
    student_religion VARCHAR(50),
    student_address TEXT,
    student_phone VARCHAR(20),
    student_email VARCHAR(255),
    
    -- Parent Data
    parent_name VARCHAR(255) NOT NULL,
    parent_nik VARCHAR(20) NOT NULL,
    parent_phone VARCHAR(20) NOT NULL,
    parent_occupation VARCHAR(100),
    parent_income VARCHAR(50),
    
    -- Previous School Data
    previous_school_name VARCHAR(255),
    previous_school_npsn VARCHAR(20),
    previous_school_address TEXT,
    
    -- Path-specific data (JSONB for flexibility)
    path_data JSONB,
    
    -- Selection
    selection_score DOUBLE PRECISION,
    ranking INTEGER,
    
    -- Status
    status VARCHAR(20) DEFAULT 'draft' CHECK (status IN (
        'draft', 'submitted', 'verified', 'rejected', 
        'accepted', 'enrolled', 'expired'
    )),
    rejection_reason TEXT,
    admin_notes TEXT,
    
    -- Timestamps
    submitted_at TIMESTAMPTZ,
    verified_at TIMESTAMPTZ,
    verified_by INTEGER REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
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
