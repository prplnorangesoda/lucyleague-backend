INSERT INTO users(steamid, username, is_admin)
VALUES ($1, $2, false)
RETURNING $table_fields;