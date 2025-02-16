-- Your SQL goes here
CREATE VIEW user_sessions AS
SELECT
    u.id AS user_id,
    u.username,
    u.email,
    s.token,
    s.expires_at,
    s.created_at AS session_created_at
FROM
    users u
JOIN
    sessions s ON u.id = s.user_id;