-- Migration 0001: Initial Schema for PG CRM (PostgreSQL)

-- Enable UUID extension if not enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users Table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'guest' CHECK (role IN ('admin', 'caretaker', 'guest')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Rooms Table
CREATE TABLE IF NOT EXISTS rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_number VARCHAR(50) UNIQUE NOT NULL,
    capacity INT NOT NULL DEFAULT 1 CHECK (capacity >= 1),
    occupied INT NOT NULL DEFAULT 0 CHECK (occupied >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Guests Table
CREATE TABLE IF NOT EXISTS guests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    room_number VARCHAR(50) NOT NULL,
    phone VARCHAR(20),
    check_in_date TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    check_out_date TIMESTAMPTZ,
    monthly_rent NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    advance_amount NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    amount_due NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for frequent queries
CREATE INDEX IF NOT EXISTS idx_guests_room_number ON guests(room_number);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
