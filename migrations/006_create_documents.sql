-- Create documents table
CREATE TABLE documents (
    id SERIAL PRIMARY KEY,
    registration_id INTEGER NOT NULL REFERENCES registrations(id) ON DELETE CASCADE,
    document_type VARCHAR(50) NOT NULL CHECK (document_type IN (
        'kartu_keluarga', 'akta_kelahiran', 'birth_certificate', 'family_card',
        'ijazah', 'rapor', 'sertifikat_prestasi', 'surat_keterangan_pindah',
        'surat_keterangan_afirmasi', 'foto', 'other'
    )),
    file_url TEXT NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_size BIGINT,
    mime_type VARCHAR(100),
    verification_status VARCHAR(20) DEFAULT 'pending' CHECK (verification_status IN (
        'pending', 'approved', 'rejected'
    )),
    verification_notes TEXT,
    verified_by INTEGER REFERENCES users(id),
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_documents_registration_id ON documents(registration_id);
CREATE INDEX idx_documents_verification_status ON documents(verification_status);

-- Create trigger for updated_at
CREATE TRIGGER update_documents_updated_at BEFORE UPDATE ON documents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
