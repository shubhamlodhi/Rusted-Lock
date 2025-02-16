-- Your SQL goes here
-- Enable necessary extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "citext";

-- -- Custom types for better data organization and validation
-- CREATE TYPE user_role AS ENUM ('user', 'admin', 'moderator', 'support');
-- CREATE TYPE auth_provider AS ENUM ('email', 'google', 'github', 'apple');
-- CREATE TYPE account_status AS ENUM ('active', 'inactive', 'suspended', 'pending_verification');
-- CREATE TYPE mfa_type AS ENUM ('authenticator', 'sms', 'email');

-- Base users table with core user information
CREATE TABLE users (
                       id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    -- Using UUID instead of SERIAL for better security and scalability

                       email CITEXT UNIQUE NOT NULL,
    -- CITEXT ensures case-insensitive email comparisons
    -- UNIQUE constraint prevents duplicate emails

                       username VARCHAR(50) UNIQUE NOT NULL,
    -- Separate username field for display purposes

                       password_hash VARCHAR(255) NOT NULL,
    -- Store Argon2 or similar modern hashing algorithm output
    -- 255 characters allows for future hash algorithm changes

                       full_name VARCHAR(100),
    -- Optional full name field

                       role VARCHAR(100) NOT NULL DEFAULT 'user',
    -- User's role in the system

                       status VARCHAR(100) NOT NULL DEFAULT 'pending_verification',
    -- Account status for managing user access

                       login_attempts SMALLINT NOT NULL DEFAULT 0,
    -- Track failed login attempts for security

                       last_login_at TIMESTAMPTZ,
    -- Track last successful login time

                       password_changed_at TIMESTAMPTZ,
    -- Track last password change for security policies

                       created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                       updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                       deleted_at TIMESTAMPTZ,
    -- Audit timestamps with soft delete support

                       CONSTRAINT username_length CHECK (char_length(username) >= 3),
                       CONSTRAINT username_format CHECK (username ~ '^[a-zA-Z0-9_-]+$')
    -- Ensure username meets minimum requirements
);