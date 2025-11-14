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
