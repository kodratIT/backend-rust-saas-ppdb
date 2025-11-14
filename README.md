# PPDB Backend - Rust + Axum

Backend API untuk Sistem PPDB (Penerimaan Peserta Didik Baru) menggunakan Rust, Axum, dan PostgreSQL.

## Tech Stack

- **Framework**: Axum 0.7
- **Database**: PostgreSQL (Supabase)
- **Cache**: Redis
- **Authentication**: JWT
- **ORM**: SQLx (compile-time checked queries)

## Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- PostgreSQL 15+
- Redis 7+

## Setup

1. Clone repository dan masuk ke folder backend:
```bash
cd ppdb-sekolah/backend
```

2. Copy environment variables:
```bash
cp .env.example .env
```

3. Edit `.env` dengan konfigurasi yang sesuai

4. Install SQLx CLI:
```bash
cargo install sqlx-cli --no-default-features --features postgres
```

5. Run database migrations:
```bash
sqlx migrate run
```

6. Build project:
```bash
cargo build
```

7. Run development server:
```bash
cargo run
```

Server akan berjalan di `http://localhost:3000`

## Development

### Run with auto-reload (using cargo-watch):
```bash
cargo install cargo-watch
cargo watch -x run
```

### Run tests:
```bash
cargo test
```

### Check code:
```bash
cargo clippy
cargo fmt --check
```

### Database migrations:
```bash
# Create new migration
sqlx migrate add <migration_name>

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

## Project Structure

```
backend/
├── src/
│   ├── main.rs              # Entry point
│   ├── config.rs            # Configuration
│   ├── api/                 # API handlers
│   ├── services/            # Business logic
│   ├── repositories/        # Data access
│   ├── models/              # Domain models
│   ├── dto/                 # Data transfer objects
│   ├── utils/               # Utilities
│   └── integrations/        # External services
├── migrations/              # Database migrations
└── tests/                   # Integration tests
```

## API Documentation

API documentation available at: `http://localhost:3000/docs` (coming soon)

## License

Proprietary - All rights reserved
