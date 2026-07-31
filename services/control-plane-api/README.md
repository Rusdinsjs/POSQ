# POSQ Server - Control Plane

Server control plane untuk aplikasi POSQ. Server ini bertanggung jawab atas:

- Autentikasi dan manajemen pengguna
- Aktivasi dan manajemen device
- Manajemen lisensi dan langganan
- Update metadata
- Backup metadata
- Admin dashboard

## Arsitektur

```
┌─────────────────────────────────────────────────────────────┐
│                      Admin Dashboard                        │
│                     (SvelteKit + Vite)                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Control Plane API                        │
│                      (Rust + Axum)                          │
│  • Authentication  • Device Activation  • License Signing   │
│  • Subscriptions   • Update Service     • Backup Metadata   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Control Plane Worker                      │
│                      (Rust + Tokio)                         │
│  • Renewal Reminders  • Cleanup Jobs  • Scheduled Tasks     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      PostgreSQL                             │
│                   (posq_server database)                    │
└─────────────────────────────────────────────────────────────┘
```

## Prerequisites

- Docker & Docker Compose
- Rust 1.78+ (untuk development)
- Node.js 20+ (untuk admin dashboard)

## Quick Start

### 1. Clone dan Setup Environment

```bash
# Copy environment file
cp .env.example .env

# Generate JWT secret
openssl rand -base64 64

# Update JWT_SECRET di .env dengan hasil generate
```

### 2. Generate Signing Keys

```bash
# Buat direktori keys
mkdir -p keys

# Generate License Signing Key (Ed25519)
openssl genpkey -algorithm Ed25519 -out keys/license_private.pem
openssl pkey -in keys/license_private.pem -pubout -out keys/license_public.pem

# Generate Update Signing Key (Ed25519) - HARUS BERBEDA dari license key!
openssl genpkey -algorithm Ed25519 -out keys/update_private.pem
openssl pkey -in keys/update_private.pem -pubout -out keys/update_public.pem

# Set permissions
chmod 600 keys/*.pem
```

### 3. Jalankan dengan Docker Compose

```bash
# Build dan jalankan semua services
docker-compose up -d

# Lihat logs
docker-compose logs -f

# Cek status
docker-compose ps
```

### 4. Akses Services

- **API Server**: http://localhost:3000
- **Admin Dashboard**: http://localhost:8080
- **PostgreSQL**: localhost:5432

### 5. Health Check

```bash
# Cek API health
curl http://localhost:3000/api/v1/health
```

## Development

### Local Development

```bash
# Install dependencies (admin dashboard)
cd services/admin-dashboard
npm install

# Jalankan API server
cd services/control-plane-api
cargo run

# Jalankan worker (terminal terpisah)
cargo run --bin control-plane-worker

# Jalankan admin dashboard
cd services/admin-dashboard
npm run dev
```

### Database Migrations

Migrations otomatis dijalankan saat API server start. Untuk menjalankan manual:

```bash
# Menggunakan sqlx CLI
cargo install sqlx-cli
cd services/control-plane-api
sqlx migrate run
```

### Testing

```bash
# Run all tests
cd services/control-plane-api
cargo test

# Run specific test
cargo test test_name
```

## API Endpoints

### Public Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/health` | Health check |

### Authentication Required

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/auth/login` | User login |
| POST | `/api/v1/auth/refresh` | Refresh access token |
| POST | `/api/v1/auth/logout` | User logout |

### Device Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/devices/activation-challenge` | Create activation challenge |
| POST | `/api/v1/devices/activate` | Activate device |
| POST | `/api/v1/devices/heartbeat` | Device heartbeat |

### License Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/licenses/refresh` | Refresh license token |

### Subscription Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/subscriptions/manual-renewal` | Manual subscription renewal |

### Update Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/updates/check` | Check for updates |
| POST | `/api/v1/updates/publish` | Publish update (admin) |

### Backup Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/backups/metadata` | Upload backup metadata |

### Admin Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/admin/merchants` | List all merchants |
| GET | `/api/v1/admin/merchants/:id` | Get merchant details |
| POST | `/api/v1/admin/devices/:id/revoke` | Revoke device |
| GET | `/api/v1/admin/audit-logs` | List audit logs |

## Security

### License Signing

- Private key TIDAK boleh ada di client binary
- Desktop hanya menerima public key
- Token ditandatangani dengan Ed25519
- Support key rotation

### Authentication

- Password di-hash dengan Argon2id
- JWT tokens dengan access dan refresh
- Rate limiting pada login attempts
- Account lockout setelah 5 failed attempts

### Tenant Isolation

- Setiap query harus filter merchant_id
- Cross-tenant access diblokir
- Admin access di-audit

## Troubleshooting

### Database Connection Issues

```bash
# Cek PostgreSQL container
docker-compose logs postgres

# Reset database
docker-compose down -v
docker-compose up -d
```

### API Server Issues

```bash
# Cek API logs
docker-compose logs control-plane-api

# Restart API
docker-compose restart control-plane-api
```

### Permission Issues

```bash
# Fix key permissions
chmod 600 keys/*.pem
chown 1000:1000 keys/*.pem
```

## Production Deployment

### Environment Variables

Pastikan semua environment variable di `.env` sudah dikonfigurasi:

- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: Secret key untuk JWT signing
- `LICENSE_SIGNING_KEY_PATH`: Path ke license private key
- `UPDATE_SIGNING_KEY_PATH`: Path ke update private key

### Backup Strategy

1. **Database Backup**: Gunakan `pg_dump` secara berkala
2. **Key Backup**: Backup signing keys ke secure storage
3. **Config Backup**: Backup `.env` file ke secure location

### Monitoring

- Health check endpoint: `/api/v1/health`
- Structured logging dalam format JSON
- Error tracking dengan tracing

## License

Proprietary - POSQ Team
