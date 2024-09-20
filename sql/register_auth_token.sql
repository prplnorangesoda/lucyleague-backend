INSERT INTO authorizations(userid, token, created_at, expires)
VALUES ($1, $2, $3, $4)
RETURNING $fields;