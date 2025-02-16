-- Your SQL goes here
-- migrations/2025-02-11-132445_create_trigger_reset_login_attempts/up.sql

CREATE TRIGGER reset_login_attempts_trigger
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION reset_login_attempts();