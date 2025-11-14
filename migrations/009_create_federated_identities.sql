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
