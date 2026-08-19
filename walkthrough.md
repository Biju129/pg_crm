# Walkthrough: PG CRM Migration to Rust & PostgreSQL

We have successfully redesigned and converted **PG CRM** from Node.js + Express + MongoDB into a **Rust-based Desktop Application with PostgreSQL**.

---

## 📁 Redesigned Project Architecture

```
pg_crm/
├── Cargo.toml                    # Rust manifest & dependencies (tokio, sqlx, argon2, jsonwebtoken, serde)
├── .env                          # Local database environment config
├── .env.example                  # Environment configuration template
├── migrations/
│   └── 0001_init.sql             # PostgreSQL tables (users, rooms, guests), indexes & constraints
├── src/
│   ├── main.rs                   # App entry point initializing logging & database pool
│   ├── config.rs                 # Environment configuration parser
│   ├── db/
│   │   ├── mod.rs
│   │   └── postgres.rs           # PostgreSQL Connection pool (sqlx::PgPool)
│   ├── models/
│   │   ├── mod.rs
│   │   ├── user.rs               # User struct, Roles, & Auth DTOs
│   │   ├── guest.rs              # Guest struct & DTOs
│   │   └── room.rs               # Room struct & DTOs
│   ├── repository/
│   │   ├── mod.rs
│   │   ├── user_repo.rs          # SQL queries for Users
│   │   ├── guest_repo.rs         # SQL queries for Guests
│   │   └── room_repo.rs          # SQL queries for Rooms
│   ├── services/
│   │   ├── mod.rs
│   │   ├── auth_service.rs       # Argon2 password hashing & JWT generation
│   │   ├── guest_service.rs      # Guest business logic
│   │   └── room_service.rs       # Room capacity & occupancy management
│   └── app/
│       ├── mod.rs
│       └── desktop.rs            # Desktop App State & API dispatchers
└── ui/                           # Desktop User Interface
    ├── index.html                # Modern tabbed Desktop layout (Guests, Rooms, Auth)
    ├── styles.css                # Dark theme desktop styling & responsiveness
    └── app.js                    # UI logic and reactive state management
```

---

## 🗄️ PostgreSQL Database Schema (`migrations/0001_init.sql`)

1. **`users` Table**:
   - `id`: `UUID PRIMARY KEY DEFAULT gen_random_uuid()`
   - `name`: `VARCHAR(100)`
   - `email`: `VARCHAR(255) UNIQUE`
   - `password_hash`: `VARCHAR(255)` (Argon2 Hashed)
   - `role`: `'admin'`, `'caretaker'`, or `'guest'`

2. **`guests` Table**:
   - `id`: `UUID PRIMARY KEY DEFAULT gen_random_uuid()`
   - `name`: `VARCHAR(100)`
   - `room_number`: `VARCHAR(50)`
   - `phone`: `VARCHAR(20)`
   - `check_in_date`, `check_out_date`: `TIMESTAMPTZ`
   - `monthly_rent`, `advance_amount`, `amount_due`: `NUMERIC(10,2)`

3. **`rooms` Table**:
   - `id`: `UUID PRIMARY KEY DEFAULT gen_random_uuid()`
   - `room_number`: `VARCHAR(50) UNIQUE`
   - `capacity`, `occupied`: `INT`

---

## 🚀 How to Run the Desktop App

1. **PostgreSQL Setup**:
   Ensure PostgreSQL is running locally on port 5432 and execute the migration:
   ```bash
   psql -U postgres -d pg_crm_db -f migrations/0001_init.sql
   ```

2. **Compile and Launch Rust App**:
   ```bash
   cargo run
   ```
