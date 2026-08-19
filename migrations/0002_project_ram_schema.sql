-- Migration 0002: Project RAM V1 Updated Database Schema Specification

-- Enable UUID extension if not enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 1. rooms Table
CREATE TABLE IF NOT EXISTS rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_number VARCHAR(50) UNIQUE NOT NULL,
    floor_number INT NOT NULL DEFAULT 1,
    capacity INT NOT NULL DEFAULT 1 CHECK (capacity >= 1),
    monthly_rent NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    status VARCHAR(20) NOT NULL DEFAULT 'AVAILABLE' 
        CHECK (status IN ('AVAILABLE', 'FULL', 'MAINTENANCE', 'INACTIVE')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2. tenants Table
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id VARCHAR(50) UNIQUE, -- Generated upon activation e.g. TNT-1001
    full_name VARCHAR(100) NOT NULL,
    contact_number VARCHAR(20) NOT NULL, -- WhatsApp / mobile
    email VARCHAR(255),
    joining_date DATE NOT NULL,
    occupation_type VARCHAR(50), -- Student / Working / Other
    organization_name VARCHAR(100), -- College / company
    room_id UUID REFERENCES rooms(id) NOT NULL,
    monthly_rent NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    advance_amount NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    enrollment_status VARCHAR(30) NOT NULL DEFAULT 'PENDING_PAYMENT' 
        CHECK (enrollment_status IN ('PENDING_PAYMENT', 'ACTIVE', 'CANCELLED')),
    joining_payment_completed BOOLEAN NOT NULL DEFAULT FALSE,
    status VARCHAR(30) NOT NULL DEFAULT 'ACTIVE' 
        CHECK (status IN ('ACTIVE', 'VACATE_REQUESTED', 'VACATED')),
    activated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 3. tenant_documents Table
CREATE TABLE IF NOT EXISTS tenant_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE NOT NULL,
    document_type VARCHAR(50) NOT NULL 
        CHECK (document_type IN ('ID_PROOF', 'ADDRESS_PROOF', 'OTHER')),
    file_name VARCHAR(255) NOT NULL,
    file_url_or_path TEXT NOT NULL,
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 4. users Table
CREATE TABLE IF NOT EXISTS users (
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

-- 5. receipts Table
CREATE TABLE IF NOT EXISTS receipts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    receipt_number VARCHAR(100) UNIQUE NOT NULL,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE NOT NULL,
    rent_ledger_id UUID,
    payment_transaction_id UUID,
    payment_method VARCHAR(20) NOT NULL CHECK (payment_method IN ('ONLINE', 'CASH')),
    amount NUMERIC(10, 2) NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    receipt_file_url TEXT,
    issued_by UUID REFERENCES users(id)
);

-- 6. enrollment_payments Table
CREATE TABLE IF NOT EXISTS enrollment_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE NOT NULL,
    payment_type VARCHAR(30) NOT NULL CHECK (payment_type IN ('ADVANCE', 'FIRST_MONTH_RENT')),
    amount_due NUMERIC(10, 2) NOT NULL,
    amount_paid NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    payment_method VARCHAR(20) NOT NULL CHECK (payment_method IN ('CASH', 'ONLINE')),
    payment_status VARCHAR(20) NOT NULL DEFAULT 'PENDING' CHECK (payment_status IN ('PENDING', 'PAID', 'FAILED')),
    paid_at TIMESTAMPTZ,
    reference_id VARCHAR(100),
    receipt_id UUID REFERENCES receipts(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 7. rent_ledger Table
CREATE TABLE IF NOT EXISTS rent_ledger (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE NOT NULL,
    room_id UUID REFERENCES rooms(id) NOT NULL,
    billing_month DATE NOT NULL,
    due_date DATE NOT NULL,
    rent_due NUMERIC(10, 2) NOT NULL,
    amount_paid NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    pending_amount NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    payment_status VARCHAR(20) NOT NULL DEFAULT 'PENDING' CHECK (payment_status IN ('PENDING', 'PARTIAL', 'PAID', 'OVERDUE')),
    payment_method VARCHAR(20) CHECK (payment_method IN ('ONLINE', 'CASH')),
    paid_at TIMESTAMPTZ,
    last_reminder_sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add foreign key constraint for receipts.rent_ledger_id
ALTER TABLE receipts DROP CONSTRAINT IF EXISTS fk_receipts_rent_ledger;
ALTER TABLE receipts ADD CONSTRAINT fk_receipts_rent_ledger 
    FOREIGN KEY (rent_ledger_id) REFERENCES rent_ledger(id) ON DELETE SET NULL;

-- 8. payment_requests Table
CREATE TABLE IF NOT EXISTS payment_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE NOT NULL,
    rent_ledger_id UUID REFERENCES rent_ledger(id) ON DELETE SET NULL,
    enrollment_payment_id UUID REFERENCES enrollment_payments(id) ON DELETE SET NULL,
    amount NUMERIC(10, 2) NOT NULL,
    payment_reference VARCHAR(100) UNIQUE NOT NULL,
    gateway_order_id VARCHAR(100) UNIQUE,
    status VARCHAR(20) NOT NULL DEFAULT 'CREATED' CHECK (status IN ('CREATED', 'SUCCESS', 'FAILED', 'EXPIRED')),
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 9. payment_transactions Table
CREATE TABLE IF NOT EXISTS payment_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_request_id UUID REFERENCES payment_requests(id) ON DELETE CASCADE NOT NULL,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE NOT NULL,
    gateway_transaction_id VARCHAR(100) UNIQUE NOT NULL,
    amount NUMERIC(10, 2) NOT NULL,
    payment_method VARCHAR(30) NOT NULL CHECK (payment_method IN ('UPI', 'CARD', 'NETBANKING', 'OTHER')),
    gateway_status VARCHAR(50) NOT NULL,
    paid_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add foreign key constraint for receipts.payment_transaction_id
ALTER TABLE receipts DROP CONSTRAINT IF EXISTS fk_receipts_payment_transaction;
ALTER TABLE receipts ADD CONSTRAINT fk_receipts_payment_transaction 
    FOREIGN KEY (payment_transaction_id) REFERENCES payment_transactions(id) ON DELETE SET NULL;

-- 10. notifications Table
CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE NOT NULL,
    notification_type VARCHAR(30) NOT NULL CHECK (notification_type IN ('WELCOME', 'RENT_DUE', 'RENT_REMINDER', 'PAYMENT_SUCCESS')),
    channel VARCHAR(20) NOT NULL CHECK (channel IN ('WHATSAPP', 'SMS', 'EMAIL')),
    message_reference VARCHAR(100),
    status VARCHAR(20) NOT NULL DEFAULT 'QUEUED' CHECK (status IN ('QUEUED', 'SENT', 'DELIVERED', 'FAILED')),
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 11. vacate_requests Table
CREATE TABLE IF NOT EXISTS vacate_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE NOT NULL,
    requested_date DATE NOT NULL,
    planned_vacate_date DATE NOT NULL,
    notice_period_days INT NOT NULL DEFAULT 30,
    status VARCHAR(30) NOT NULL DEFAULT 'REQUESTED' CHECK (status IN ('REQUESTED', 'UNDER_REVIEW', 'APPROVED', 'REJECTED', 'COMPLETED')),
    admin_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 12. vacate_settlements Table
CREATE TABLE IF NOT EXISTS vacate_settlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vacate_request_id UUID REFERENCES vacate_requests(id) ON DELETE CASCADE NOT NULL,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE NOT NULL,
    advance_amount NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    pending_rent_deduction NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    damage_deduction NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    other_deduction NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    total_deduction NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    refund_amount NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    inspection_completed BOOLEAN NOT NULL DEFAULT FALSE,
    keys_returned BOOLEAN NOT NULL DEFAULT FALSE,
    items_returned BOOLEAN NOT NULL DEFAULT FALSE,
    refund_status VARCHAR(20) NOT NULL DEFAULT 'PENDING' CHECK (refund_status IN ('PENDING', 'APPROVED', 'PAID')),
    refund_payment_reference VARCHAR(100),
    approved_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    notes TEXT
);

-- 13. audit_logs Table
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    action VARCHAR(50) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID,
    old_data JSONB,
    new_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for frequent queries
CREATE INDEX IF NOT EXISTS idx_tenants_tenant_id ON tenants(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenants_room_id ON tenants(room_id);
CREATE INDEX IF NOT EXISTS idx_tenants_enrollment_status ON tenants(enrollment_status);
CREATE INDEX IF NOT EXISTS idx_tenants_status ON tenants(status);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username_or_login);
CREATE INDEX IF NOT EXISTS idx_rent_ledger_tenant_due ON rent_ledger(tenant_id, due_date);
CREATE INDEX IF NOT EXISTS idx_rent_ledger_status ON rent_ledger(payment_status);
CREATE INDEX IF NOT EXISTS idx_enrollment_payments_tenant ON enrollment_payments(tenant_id);
CREATE INDEX IF NOT EXISTS idx_payment_requests_ref ON payment_requests(payment_reference);
CREATE INDEX IF NOT EXISTS idx_notifications_tenant ON notifications(tenant_id);
CREATE INDEX IF NOT EXISTS idx_vacate_requests_tenant ON vacate_requests(tenant_id);
