# 📚 PPDB API Documentation

## Quick Start

### 1. Install Dependencies

```bash
cd ppdb-sekolah/backend
cargo build
```

### 2. Run Server

```bash
cargo run
```

### 3. Access Documentation

Open your browser and visit:

#### 🎨 **Swagger UI** (Interactive Testing)
```
http://localhost:8000/api/docs/swagger
```
**Best for:** Testing API endpoints interactively

#### 🚀 **RapiDoc** (Modern UI)
```
http://localhost:8000/api/docs/rapidoc
```
**Best for:** Modern, fast interface with dark mode

#### 📚 **ReDoc** (Clean Documentation)
```
http://localhost:8000/api/docs/redoc
```
**Best for:** Reading documentation, printing

#### 📄 **OpenAPI Spec** (JSON)
```
http://localhost:8000/api/docs/openapi.json
```
**Best for:** Import to Postman, generate clients

---

## 🎯 Features

✅ **Interactive API Testing** - Test endpoints directly from browser  
✅ **Auto-Generated** - Documentation always in sync with code  
✅ **Type-Safe** - Compile-time validation  
✅ **Multiple UIs** - Swagger, RapiDoc, ReDoc  
✅ **JWT Authentication** - Built-in auth support  
✅ **Request/Response Examples** - See real data examples  
✅ **Export to Postman** - One-click import  
✅ **Client Generation** - Generate TypeScript, Python, etc.

---

## 🔐 Authentication

Most endpoints require JWT Bearer token.

### How to Authenticate:

1. **Register or Login** via `/api/v1/auth/login`
2. **Copy the access_token** from response
3. **Click "Authorize"** button in Swagger UI
4. **Enter:** `Bearer <your_token>`
5. **Test endpoints** with authentication

### Example:

```bash
# 1. Login
curl -X POST http://localhost:8000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "password123"
  }'

# Response:
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400
}

# 2. Use token in subsequent requests
curl -X GET http://localhost:8000/api/v1/users/me \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

---

## 📖 API Overview

### Authentication
- `POST /api/v1/auth/register` - Register new user
- `POST /api/v1/auth/login` - Login user
- `POST /api/v1/auth/verify-email` - Verify email
- `POST /api/v1/auth/logout` - Logout user
- `GET /api/v1/auth/me` - Get current user

### Schools (SuperAdmin only)
- `GET /api/v1/schools` - List all schools
- `POST /api/v1/schools` - Create school
- `GET /api/v1/schools/{id}` - Get school details
- `PUT /api/v1/schools/{id}` - Update school
- `DELETE /api/v1/schools/{id}` - Delete school

### Users
- `GET /api/v1/users` - List users (filtered by school)
- `POST /api/v1/users` - Create user
- `GET /api/v1/users/{id}` - Get user details
- `PUT /api/v1/users/{id}` - Update user
- `DELETE /api/v1/users/{id}` - Delete user
- `GET /api/v1/users/me` - Get current user profile
- `PUT /api/v1/users/me` - Update current user profile

### Periods
- `GET /api/v1/periods` - List periods
- `POST /api/v1/periods` - Create period with paths
- `GET /api/v1/periods/{id}` - Get period details
- `PUT /api/v1/periods/{id}` - Update period
- `DELETE /api/v1/periods/{id}` - Delete period
- `POST /api/v1/periods/{id}/activate` - Activate period
- `POST /api/v1/periods/{id}/close` - Close period

### Registrations
- `GET /api/v1/registrations` - List registrations
- `POST /api/v1/registrations` - Create registration (draft)
- `GET /api/v1/registrations/{id}` - Get registration details
- `PUT /api/v1/registrations/{id}` - Update registration
- `POST /api/v1/registrations/{id}/submit` - Submit registration
- `POST /api/v1/registrations/{id}/verify` - Verify registration (admin)
- `POST /api/v1/registrations/{id}/reject` - Reject registration (admin)
- `GET /api/v1/registrations/pending` - Get pending verifications (admin)

### Documents
- `GET /api/v1/registrations/{id}/documents` - List documents
- `POST /api/v1/registrations/{id}/documents` - Upload document
- `DELETE /api/v1/registrations/{id}/documents/{doc_id}` - Delete document

### Selection
- `POST /api/v1/periods/{id}/calculate-scores` - Calculate scores (admin)
- `GET /api/v1/periods/{id}/rankings` - Get rankings
- `POST /api/v1/periods/{id}/run-selection` - Run selection (admin)
- `POST /api/v1/periods/{id}/announce` - Announce results (admin)
- `GET /api/v1/registrations/{id}/result` - Check result (public)

---

## 🎨 UI Screenshots

### Swagger UI
![Swagger UI](https://swagger.io/swagger/media/Images/tools/SwaggerUI.png)

### RapiDoc
![RapiDoc](https://rapidocweb.com/images/rapidoc-example.png)

### ReDoc
![ReDoc](https://redocly.com/images/redoc-example.png)

---

## 🔧 Import to Postman

### Method 1: Direct Import
1. Open Postman
2. Click **Import**
3. Select **Link**
4. Enter: `http://localhost:8000/api/docs/openapi.json`
5. Click **Continue**
6. All endpoints imported! ✅

### Method 2: File Import
```bash
# Download OpenAPI spec
curl http://localhost:8000/api/docs/openapi.json > openapi.json

# Import file to Postman
```

---

## 🚀 Generate API Clients

### TypeScript/JavaScript
```bash
npx @openapitools/openapi-generator-cli generate \
  -i http://localhost:8000/api/docs/openapi.json \
  -g typescript-axios \
  -o ./client/typescript
```

### Python
```bash
openapi-generator-cli generate \
  -i http://localhost:8000/api/docs/openapi.json \
  -g python \
  -o ./client/python
```

### Go
```bash
openapi-generator-cli generate \
  -i http://localhost:8000/api/docs/openapi.json \
  -g go \
  -o ./client/go
```

### Java
```bash
openapi-generator-cli generate \
  -i http://localhost:8000/api/docs/openapi.json \
  -g java \
  -o ./client/java
```

---

## 📝 Example Usage

### JavaScript/TypeScript
```typescript
import axios from 'axios';

const api = axios.create({
  baseURL: 'http://localhost:8000/api/v1',
});

// Login
const { data } = await api.post('/auth/login', {
  email: 'admin@example.com',
  password: 'password123',
});

// Set token
api.defaults.headers.common['Authorization'] = `Bearer ${data.access_token}`;

// Get current user
const user = await api.get('/users/me');
console.log(user.data);

// List schools
const schools = await api.get('/schools');
console.log(schools.data);
```

### Python
```python
import requests

BASE_URL = 'http://localhost:8000/api/v1'

# Login
response = requests.post(f'{BASE_URL}/auth/login', json={
    'email': 'admin@example.com',
    'password': 'password123'
})
token = response.json()['access_token']

# Set headers
headers = {'Authorization': f'Bearer {token}'}

# Get current user
user = requests.get(f'{BASE_URL}/users/me', headers=headers)
print(user.json())

# List schools
schools = requests.get(f'{BASE_URL}/schools', headers=headers)
print(schools.json())
```

### cURL
```bash
# Login
TOKEN=$(curl -X POST http://localhost:8000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"password123"}' \
  | jq -r '.access_token')

# Get current user
curl -X GET http://localhost:8000/api/v1/users/me \
  -H "Authorization: Bearer $TOKEN"

# List schools
curl -X GET http://localhost:8000/api/v1/schools \
  -H "Authorization: Bearer $TOKEN"
```

---

## 🎓 Learning Resources

- [OpenAPI Specification](https://swagger.io/specification/)
- [Swagger UI Documentation](https://swagger.io/tools/swagger-ui/)
- [utoipa Rust Crate](https://docs.rs/utoipa/)
- [RapiDoc](https://rapidocweb.com/)
- [ReDoc](https://redocly.com/)

---

## 🐛 Troubleshooting

### Documentation not loading?
```bash
# Check if server is running
curl http://localhost:8000/health

# Rebuild project
cargo clean && cargo build
```

### Endpoints missing in Swagger?
Check that endpoints are registered in `src/api/docs.rs`

### Authentication not working?
1. Make sure you clicked "Authorize" button
2. Token format: `Bearer <token>` (with space)
3. Token must not be expired (24 hours)

---

## 📞 Support

For issues or questions:
- Check [API_DOCUMENTATION_GUIDE.md](./API_DOCUMENTATION_GUIDE.md)
- Open an issue on GitHub
- Contact: support@ppdb.com

---

## ✅ Status

- ✅ Setup complete
- ✅ Swagger UI working
- ✅ RapiDoc working
- ✅ ReDoc working
- ✅ OpenAPI spec generated
- ⏳ Documenting all endpoints (in progress)
- ⏳ Adding comprehensive examples (in progress)

**Current Progress: Phase 20 - Step 1-3 Complete** 🎉
