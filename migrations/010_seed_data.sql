-- Seed data for development

-- Insert sample schools
INSERT INTO schools (name, npsn, code, address, phone, email, status) VALUES
('SMA Negeri 1 Jakarta', '20100001', 'SMAN1JKT', 'Jl. Sudirman No. 1, Jakarta Pusat', '021-1234567', 'info@sman1jkt.sch.id', 'active'),
('SMP Negeri 5 Bandung', '20200005', 'SMPN5BDG', 'Jl. Asia Afrika No. 5, Bandung', '022-7654321', 'info@smpn5bdg.sch.id', 'active'),
('SD Negeri 10 Surabaya', '20300010', 'SDN10SBY', 'Jl. Pahlawan No. 10, Surabaya', '031-9876543', 'info@sdn10sby.sch.id', 'active');

-- Insert super admin (password: admin123)
-- Hash generated with argon2 using generate_hash binary
INSERT INTO users (school_id, email, password_hash, full_name, role, email_verified) VALUES
(NULL, 'superadmin@ppdb.com', '$argon2id$v=19$m=19456,t=2,p=1$e9VbN5rXC+1/M77vebkD6Q$oPxRBCMjherBjEqv+0aVwgabo/boAu8/Rk8u6jsuXOg', 'Super Admin', 'super_admin', true);

-- Insert school admins for each school (password: admin123)
INSERT INTO users (school_id, email, password_hash, full_name, phone, role, email_verified) VALUES
(1, 'admin@sman1jkt.sch.id', '$argon2id$v=19$m=19456,t=2,p=1$e9VbN5rXC+1/M77vebkD6Q$oPxRBCMjherBjEqv+0aVwgabo/boAu8/Rk8u6jsuXOg', 'Admin SMA 1 Jakarta', '081234567890', 'school_admin', true),
(2, 'admin@smpn5bdg.sch.id', '$argon2id$v=19$m=19456,t=2,p=1$e9VbN5rXC+1/M77vebkD6Q$oPxRBCMjherBjEqv+0aVwgabo/boAu8/Rk8u6jsuXOg', 'Admin SMP 5 Bandung', '082345678901', 'school_admin', true),
(3, 'admin@sdn10sby.sch.id', '$argon2id$v=19$m=19456,t=2,p=1$e9VbN5rXC+1/M77vebkD6Q$oPxRBCMjherBjEqv+0aVwgabo/boAu8/Rk8u6jsuXOg', 'Admin SD 10 Surabaya', '083456789012', 'school_admin', true);

-- Insert sample parent users (password: admin123)
INSERT INTO users (school_id, email, password_hash, full_name, phone, nik, role, email_verified) VALUES
(1, 'parent1@example.com', '$argon2id$v=19$m=19456,t=2,p=1$e9VbN5rXC+1/M77vebkD6Q$oPxRBCMjherBjEqv+0aVwgabo/boAu8/Rk8u6jsuXOg', 'Budi Santoso', '081111111111', '3201010101010001', 'parent', true),
(2, 'parent2@example.com', '$argon2id$v=19$m=19456,t=2,p=1$e9VbN5rXC+1/M77vebkD6Q$oPxRBCMjherBjEqv+0aVwgabo/boAu8/Rk8u6jsuXOg', 'Siti Rahayu', '082222222222', '3202020202020002', 'parent', true);

-- Insert sample periods
INSERT INTO periods (school_id, academic_year, level, start_date, end_date, registration_start, registration_end, announcement_date, reenrollment_deadline, status) VALUES
(1, '2025/2026', 'SMA', '2025-07-01', '2026-06-30', '2025-01-01', '2025-02-28', '2025-03-15', '2025-04-01', 'active'),
(2, '2025/2026', 'SMP', '2025-07-01', '2026-06-30', '2025-01-01', '2025-02-28', '2025-03-15', '2025-04-01', 'active'),
(3, '2025/2026', 'SD', '2025-07-01', '2026-06-30', '2025-01-01', '2025-02-28', '2025-03-15', '2025-04-01', 'active');

-- Insert registration paths for each period
INSERT INTO registration_paths (period_id, path_type, name, quota, description, scoring_config) VALUES
-- SMA Negeri 1 Jakarta
(1, 'zonasi', 'Jalur Zonasi', 100, 'Jalur Zonasi - berdasarkan jarak tempat tinggal', '{"max_distance_km": 5, "weight": 1.0}'),
(1, 'prestasi', 'Jalur Prestasi', 50, 'Jalur Prestasi - berdasarkan nilai rapor dan prestasi', '{"rapor_weight": 0.6, "achievement_weight": 0.4}'),
(1, 'afirmasi', 'Jalur Afirmasi', 20, 'Jalur Afirmasi - untuk siswa kurang mampu', '{"criteria": "economic_status"}'),
(1, 'perpindahan_tugas', 'Jalur Perpindahan Tugas Orang Tua', 10, 'Jalur Perpindahan Tugas Orang Tua', '{"document_required": "surat_tugas"}'),

-- SMP Negeri 5 Bandung
(2, 'zonasi', 'Jalur Zonasi', 80, 'Jalur Zonasi - berdasarkan jarak tempat tinggal', '{"max_distance_km": 3, "weight": 1.0}'),
(2, 'prestasi', 'Jalur Prestasi', 40, 'Jalur Prestasi - berdasarkan nilai rapor dan prestasi', '{"rapor_weight": 0.7, "achievement_weight": 0.3}'),
(2, 'afirmasi', 'Jalur Afirmasi', 15, 'Jalur Afirmasi - untuk siswa kurang mampu', '{"criteria": "economic_status"}'),

-- SD Negeri 10 Surabaya
(3, 'zonasi', 'Jalur Zonasi', 60, 'Jalur Zonasi - berdasarkan jarak tempat tinggal', '{"max_distance_km": 2, "weight": 1.0}'),
(3, 'prestasi', 'Jalur Prestasi', 20, 'Jalur Prestasi - berdasarkan prestasi TK', '{"tk_report_weight": 1.0}'),
(3, 'afirmasi', 'Jalur Afirmasi', 10, 'Jalur Afirmasi - untuk siswa kurang mampu', '{"criteria": "economic_status"}');
