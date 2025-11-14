# 🎓 PPDB Backend - Rust + Axum

Backend API untuk Sistem Penerimaan Peserta Didik Baru (PPDB) menggunakan Rust, Axum, dan PostgreSQL dengan arsitektur multi-tenant.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/axum-0.7-blue.svg)](https://github.com/tokio-rs/axum)
[![PostgreSQL](https://img.shields.io/badge/postgresql-14%2B-blue.svg)](https://www.postgresql.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🚀 Tech Stack

- **Framework**: Axum (High-performance Rust web framework)
- **Database**: PostgreSQL with Row-Level Security (Supabase)
- **Authentication**: JWT with refresh tokens (24h access, 7d refresh)
- **Password Hashing**: Argon2 (industry standard)
- **Validation**: validator crate with custom rules
- **ORM**: SQLx (compile-time checked queries)
- **Runtime**: Tokio (async runtime)
- **Logging**: tracing + tracing-subscriber

## ✨ Features

### ✅ Implemented (Phase 3 & 4)
- **JWT Authentication** with access & refresh tokens
- **Role-Based Access Control** (super_admin, school_admin, parent)
- **Multi-tenant Architecture** with PostgreSQL RLS
- **Email Verification** flow
- **Password Reset** with expiring tokens
- **School Management** (Super Admin only)
- **User Management** with tenant isolation
- **Middleware Stack**: Auth, RBAC, Tenant Context
- **Comprehensive Error Handling**
- **Input Validation** with custom validators

### 🚧 Coming Soon (Phase 5-19)
- Period & Registration Path Management
- Student Registration & Document Upload
- Document Verification Flow
- Selection Scoring & Ranking
- Payment Integration (Midtrans)
- Re-enrollment Process
- Dashboard & Reporting
- Audit Logging
- Federated Identity & SSO (Keycloak)
- External System Integration
- Security Enhancements (Rate Limiting, 2FA)
- Performance Optimization (Redis Caching)

## 📋 Prerequisites

- **Rust** 1.70+ ([Install from rustup.rs](https://rustup.rs))
- **PostgreSQL** 14+ or Supabase account
- **Redis** (optional, for caching)
- **Git**

## 🛠️ Installation

### 1. Clone Repository
```bash
git clone https://github.com/kodratIT/backend-rust-saas-ppdb.git
cd backend-rust-saas-ppdb
```

### 2. Setup Environment
```bash
cp .env.example .env
```

Edit `.env` with your configuration:
```env
DATABASE_URL=postgresql://user:password@localhost:5432/ppdb
JWT_SECRET=your-super-secret-key-min-32-chars
JWT_EXPIRATION_HOURS=24
PORT=8080
RUST_LOG=debug
```

### 3. Install SQLx CLI
```bash
cargo install sqlx-cli --no-default-features --features postgres
```

### 4. Run Migrations
```bash
sqlx migrate run
```

### 5. Start Development Server
```bash
cargo run
```

Server akan berjalan di `http://localhost:8080` 🎉

## 📚 API Documentation

### Base URL
```
http://localhost:8080/api/v1
```

### Authentication Endpoints

#### Register User
```http
POST /auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123",
  "full_name": "John Doe",
  "phone": "081234567890",
  "nik": "1234567890123456"
}
```

#### Login
```http
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}

Response:
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "id": 1,
    "email": "user@example.com",
    "full_name": "John Doe",
    "role": "parent",
    "school_id": null
  }
}
```

#### Refresh Token
```http
POST /auth/refresh
Content-Type: application/json

{
  "refresh_token": "eyJ..."
}
```

#### Get Current User
```http
GET /auth/me
Authorization: Bearer <access_token>
```

#### Logout
```http
POST /auth/logout
Authorization: Bearer <access_token>
```

### School Management (Super Admin Only)

#### List Schools
```http
GET /schools?page=1&page_size=10&search=&status=active
Authorization: Bearer <access_token>
```

#### Create School
```http
POST /schools
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "name": "SMA Negeri 1",
  "npsn": "12345678",
  "code": "SMAN1",
  "address": "Jl. Pendidikan No. 1",
  "phone": "021-1234567",
  "email": "info@sman1.sch.id"
}
```

### User Management

#### List Users
```http
GET /users?page=1&page_size=10&role=parent
Authorization: Bearer <access_token>
```

#### Create User (Admin Only)
```http
POST /users
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "email": "admin@school.com",
  "password": "password123",
  "full_name": "School Admin",
  "role": "school_admin",
  "school_id": 1
}
```

#### Change Password
```http
POST /users/me/change-password
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "old_password": "oldpass123",
  "new_password": "newpass123"
}
```

## 🏗️ Project Structure

```
backend/
├── migrations/              # Database migrations (SQLx)
│   ├── 001_create_schools.sql
│   ├── 002_create_users.sql
│   └── ...
├── src/
│   ├── api/                # API routes & handlers
│   │   ├── middleware/     # Auth, RBAC, Tenant context
│   │   ├── auth.rs         # Authentication endpoints
│   │   ├── schools.rs      # School management
│   │   └── users.rs        # User management
│   ├── dto/                # Data Transfer Objects
│   ├── integrations/       # External services (Resend, Midtrans, etc)
│   ├── models/             # Database models
│   ├── repositories/       # Data access layer
│   ├── services/           # Business logic layer
│   ├── utils/              # Utilities (JWT, password, errors)
│   ├── config.rs           # Configuration management
│   └── main.rs             # Application entry point
├── tests/                  # Integration tests
├── Cargo.toml              # Dependencies
├── .env.example            # Environment template
└── README.md
```

## 🧪 Development

### Run Tests
```bash
cargo test
```

### Format Code
```bash
cargo fmt
```

### Lint Code
```bash
cargo clippy -- -D warnings
```

### Watch Mode (Auto-reload)
```bash
cargo install cargo-watch
cargo watch -x run
```

### Database Commands

Create migration:
```bash
sqlx migrate add <migration_name>
```

Run migrations:
```bash
sqlx migrate run
```

Revert migration:
```bash
sqlx migrate revert
```

## 🚢 Deployment

### Fly.io Deployment

1. Install Fly CLI:
```bash
curl -L https://fly.io/install.sh | sh
```

2. Login:
```bash
fly auth login
```

3. Launch app:
```bash
fly launch
```

4. Set secrets:
```bash
fly secrets set DATABASE_URL="postgresql://..."
fly secrets set JWT_SECRET="your-secret"
```

5. Deploy:
```bash
fly deploy
```

## 🔐 Security Features

| Feature | Status | Description |
|---------|--------|-------------|
| Password Hashing | ✅ | Argon2 with salt |
| JWT Authentication | ✅ | Access + Refresh tokens |
| RBAC | ✅ | Role-based access control |
| Multi-tenant Isolation | ✅ | PostgreSQL RLS |
| Email Verification | ✅ | Token-based verification |
| Password Reset | ✅ | Expiring reset tokens |
| Input Validation | ✅ | Comprehensive validation |
| Rate Limiting | 🚧 | Coming in Phase 16 |
| 2FA | 🚧 | Coming in Phase 16 |
| CAPTCHA | 🚧 | Coming in Phase 16 |

## 🗺️ Roadmap

- [x] **Phase 1**: Project Setup & Infrastructure
- [x] **Phase 2**: Database Schema & Multi-Tenant Foundation
- [x] **Phase 3**: Authentication & Authorization ✨
- [x] **Phase 4**: School & User Management ✨
- [ ] **Phase 5**: Period & Registration Path Management
- [ ] **Phase 6**: Student Registration & Document Upload
- [ ] **Phase 7**: Document Verification
- [ ] **Phase 8**: Selection Scoring & Ranking
- [ ] **Phase 9**: Final Selection & Announcement
- [ ] **Phase 10**: Payment Integration (Midtrans)
- [ ] **Phase 11**: Re-enrollment Process
- [ ] **Phase 12**: Dashboard & Reporting
- [ ] **Phase 13**: Audit Logging
- [ ] **Phase 14**: Federated Identity & SSO
- [ ] **Phase 15**: External System Integration
- [ ] **Phase 16**: Security Enhancements
- [ ] **Phase 17**: Performance Optimization
- [ ] **Phase 18**: Error Handling & Logging
- [ ] **Phase 19**: Documentation & Deployment

## 🤝 Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👨‍💻 Author

**Kodrat IT**
- GitHub: [@kodratIT](https://github.com/kodratIT)

## 🙏 Acknowledgments

- Built with [Axum](https://github.com/tokio-rs/axum)
- Database by [Supabase](https://supabase.com)
- Inspired by modern SaaS architectures

---

⭐ Star this repo if you find it helpful!
