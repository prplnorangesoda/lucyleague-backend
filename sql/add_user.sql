INSERT INTO users(steamid, username)
VALUES ($1, $2)
RETURNING $table_fields;