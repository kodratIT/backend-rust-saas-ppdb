# Setup Guide - PPDB Backend

Panduan lengkap untuk setup dan menjalankan PPDB Backend.

## Prerequisites

1. **Rust** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **PostgreSQL** (15+)
   - Install PostgreSQL atau gunakan Supabase (recommended)
   - Buat database baru: `ppdb_db`

3. **Redis** (7+) - Optional untuk caching
   ```bash
   # macOS
   brew install redis
   brew services start redis
   
   # Ubuntu
   sudo apt install redis-server
   sudo systemctl start redis
   ```

## Quick Start

### 1. Clone & Setup

```bash
cd ppdb-sekolah/backend
make setup
```

### 2. Configure Environment

Edit `.env` file dengan konfigurasi Anda:

```bash
# Database - Gunakan Supabase atau PostgreSQL lokal
DATABASE_URL=postgresql://postgres:password@localhost:5432/ppdb_db

# JWT Secret - Generate dengan: openssl rand -base64 32
JWT_SECRET=your-super-secret-jwt-key-change-this

# Supabase (jika menggunakan Supabase)
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_ANON_KEY=your-anon-key
SUPABASE_SERVICE_KEY=your-service-key
```

### 3. Install Development Tools

```bash
make install-tools
```

### 4. Run Migrations

```bash
make migrate
```

### 5. Seed Sample Data (Optional)

```bash
make seed
```

### 6. Run Development Server

```bash
make dev
```

Server akan berjalan di: `http://localhost:3000`

## Development Workflow

### Run with auto-reload
```bash
make dev
# atau
cargo watch -x run
```

### Run tests
```bash
make test
```

### Format code
```bash
make fmt
```

### Check code quality
```bash
make check
```

### Database migrations

Create new migration:
```bash
sqlx migrate add <migration_name>
```

Run migrations:
```bash
make migrate
```

Revert last migration:
```bash
make migrate-down
```

## Testing

### Run all tests
```bash
cargo test
```

### Run specific test
```bash
cargo test test_name
```

### Run with output
```bash
cargo test -- --nocapture
```

## Sample Users (After Seeding)

### Super Admin
- Email: `superadmin@ppdb.com`
- Password: `admin123`

### School Admin (SMA Negeri 1 Jakarta)
- Email: `admin@sman1jkt.sch.id`
- Password: `admin123`

### Parent
- Email: `parent1@example.com`
- Password: `admin123`

## API Endpoints

### Health Check
```bash
curl http://localhost:3000/health
```

### Authentication (Coming Soon)
```bash
# Register
POST /api/v1/auth/register

# Login
POST /api/v1/auth/login

# Verify Email
POST /api/v1/auth/verify-email
```

## Troubleshooting

### Database connection error
- Pastikan PostgreSQL running
- Check DATABASE_URL di .env
- Test connection: `psql $DATABASE_URL`

### Migration error
- Check migration files di `migrations/`
- Revert dan run ulang: `make migrate-down && make migrate`

### Compilation error
- Update Rust: `rustup update`
- Clean build: `make clean && cargo build`

## Project Structure

```
backend/
├── src/
│   ├── main.rs              # Entry point
│   ├── config.rs            # Configuration
│   ├── api/                 # API handlers
│   │   ├── auth.rs
│   │   └── middleware/
│   ├── services/            # Business logic
│   ├── repositories/        # Data access
│   ├── models/              # Domain models
│   ├── dto/                 # Data transfer objects
│   ├── utils/               # Utilities
│   │   ├── error.rs
│   │   ├── jwt.rs
│   │   └── password.rs
│   └── integrations/        # External services
├── migrations/              # Database migrations
├── tests/                   # Integration tests
├── Cargo.toml              # Dependencies
└── .env                    # Environment variables
```

## Next Steps

1. ✅ Setup project structure
2. ✅ Database migrations
3. 🔄 Implement authentication (Phase 3)
4. 🔄 Implement user management (Phase 4)
5. 🔄 Implement registration flow (Phase 6)

## Resources

- [Axum Documentation](https://docs.rs/axum)
- [SQLx Documentation](https://docs.rs/sqlx)
- [Tokio Documentation](https://tokio.rs)
- [Rust Book](https://doc.rust-lang.org/book/)

## Support

Untuk pertanyaan atau issue, silakan buat issue di repository atau hubungi tim development.
