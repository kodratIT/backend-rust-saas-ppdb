-- Create registration_paths table
CREATE TABLE registration_paths (
    id SERIAL PRIMARY KEY,
    period_id INTEGER NOT NULL REFERENCES periods(id) ON DELETE CASCADE,
    path_type VARCHAR(50) NOT NULL CHECK (path_type IN ('zonasi', 'prestasi', 'afirmasi', 'perpindahan_tugas')),
    name VARCHAR(255) NOT NULL,
    quota INTEGER NOT NULL,
    description TEXT,
    scoring_config JSONB,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
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
