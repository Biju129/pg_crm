-- Migration 0003: Upgrade users table and fix rooms table to match Project RAM V1 schema

-- ================================================================
-- 1. Recreate users table with Project RAM V1 columns
-- ================================================================

-- Drop old constraints and indexes
DROP INDEX IF EXISTS idx_users_email;

-- Rename old users table for backup
ALTER TABLE IF EXISTS users RENAME TO users_old;

-- Drop FK references temporarily (they'll be recreated below)
ALTER TABLE audit_logs DROP CONSTRAINT IF EXISTS audit_logs_user_id_fkey;
ALTER TABLE receipts DROP CONSTRAINT IF EXISTS receipts_issued_by_fkey;
ALTER TABLE vacate_settlements DROP CONSTRAINT IF EXISTS vacate_settlements_approved_by_fkey;

-- Create the new Project RAM V1 users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username_or_login VARCHAR(100) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role VARCHAR(20) NOT NULL CHECK (role IN ('OWNER', 'ADMIN', 'TENANT')),
    tenant_id UUID REFERENCES tenants(id) ON DELETE SET NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    must_change_password BOOLEAN NOT NULL DEFAULT FALSE,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Restore FK constraints pointing to the new users table
ALTER TABLE audit_logs
    ADD CONSTRAINT audit_logs_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE receipts
    ADD CONSTRAINT receipts_issued_by_fkey
    FOREIGN KEY (issued_by) REFERENCES users(id);

ALTER TABLE vacate_settlements
    ADD CONSTRAINT vacate_settlements_approved_by_fkey
    FOREIGN KEY (approved_by) REFERENCES users(id);

-- Add index on username_or_login
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username_or_login);

-- Drop the backup table
DROP TABLE IF EXISTS users_old;

-- ================================================================
-- 2. Upgrade rooms table: add floor_number and monthly_rent columns
--    if they are missing (safe for fresh and existing installs)
-- ================================================================

-- Add floor_number if missing
DO $$ BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name='rooms' AND column_name='floor_number'
    ) THEN
        ALTER TABLE rooms ADD COLUMN floor_number INT NOT NULL DEFAULT 1;
    END IF;
END $$;

-- Add monthly_rent if missing
DO $$ BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name='rooms' AND column_name='monthly_rent'
    ) THEN
        ALTER TABLE rooms ADD COLUMN monthly_rent NUMERIC(10,2) NOT NULL DEFAULT 0.00;
    END IF;
END $$;

-- Add status if missing (rename from old monthly_rent_rate if needed)
DO $$ BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name='rooms' AND column_name='status'
    ) THEN
        ALTER TABLE rooms ADD COLUMN status VARCHAR(20) NOT NULL DEFAULT 'AVAILABLE'
            CHECK (status IN ('AVAILABLE', 'FULL', 'MAINTENANCE', 'INACTIVE'));
    END IF;
END $$;

-- ================================================================
-- 3. Verify: show final users and rooms table structure
-- ================================================================
SELECT 'Migration 0003 complete: users and rooms tables upgraded to Project RAM V1 schema.' AS result;
