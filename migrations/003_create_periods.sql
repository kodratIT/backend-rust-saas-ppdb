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
