 ## NGN Verify (Community-first fake product reporting)
 
 A production-shaped Rust backend (Axum + Postgres) for community-first product verification.
 
 You can:
 - **Sign up / log in** (JWT auth)
 - **Lookup** a product by barcode/GTIN
 - **Submit claims** (legit/suspect) against products
 - Explore and test APIs via **Swagger UI**
 
 ### Why this structure (for Python devs)
 If you’re coming from FastAPI/Flask/Django:
 - **Axum router** ≈ FastAPI routes / Flask blueprints
 - **Extractors** (`Json<T>`, `State`, headers) ≈ FastAPI dependencies / request parsing
 - **Services** hold business logic (like Django service layers)
 - **SQLx** is the DB layer (queries + migrations)
 
 ### Tech stack
 - **API**: `axum`
 - **DB**: PostgreSQL + `sqlx` migrations
 - **Auth**: Argon2 password hashing, JWT bearer tokens
 - **Docs**: OpenAPI via `utoipa`, Swagger UI via `utoipa-swagger-ui`
 
### Quick start (dev)

#### Option A: Postgres via Docker (recommended for most devs)

1) Start Postgres

```bash
docker compose up -d
```

2) Create `.env`

```bash
cp .env.example .env
```

3) Run migrations

```bash
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run
```

4) Run the API

```bash
cargo run
```

Stop Postgres when you’re done:

```bash
docker compose down
```

#### Option B: Postgres installed locally (your “normal” way)

1) Ensure Postgres is running locally and create the DB:

```bash
createdb ngn_verify
```

2) Create `.env` and point `DATABASE_URL` to your local Postgres user/password:

```bash
cp .env.example .env
```

Example `DATABASE_URL`:

```bash
DATABASE_URL=postgres://<user>:<password>@localhost:5432/ngn_verify
```

3) Run migrations and start the API:

```bash
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run
cargo run
```
 
 ### Swagger UI
 - **Swagger UI**: `http://localhost:8080/docs`
 - **OpenAPI JSON**: `http://localhost:8080/api-docs/openapi.json`
 
 In Swagger UI:
 - Call `POST /auth/signup` or `POST /auth/login`
 - Copy the returned `access_token`
 - Click **Authorize** and paste: `Bearer <token>`
 
 ### MVP endpoints
 - **Auth**
   - `POST /auth/signup`
   - `POST /auth/login`
   - `GET /me`
 - **Lookup**
   - `GET /lookup?gtin=...`
 - **Claims**
   - `POST /claims`
 
 ### Production notes (what to change before real deploy)
 - **JWT_SECRET** must be long + random (env/secret manager).
 - Put the API behind TLS (nginx, cloud LB).
 - Add rate limiting + abuse controls (IP + account based).
 - Store evidence in object storage (S3/MinIO) and only keep URLs/hashes in DB.
 - Add structured logging + metrics (this project already uses `tracing`).

 ### Database schema (MVP)
 - **users**: email/password login, role
 - **products**: product info + `community_status`
 - **product_identifiers**: GTIN/NAFDAC/QR values mapped to products
 - **claims**: user-submitted legit/suspect claim per product
