-- Your SQL goes here
-- migrations/2025-02-11-132444_reset_login_attempts/up.sql

CREATE OR REPLACE FUNCTION reset_login_attempts() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.last_login_at < NOW() - INTERVAL '1 minutes' THEN
        NEW.login_attempts := 0;
END IF;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;