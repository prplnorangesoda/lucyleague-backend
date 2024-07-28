INSERT INTO testing.users(steamid, username)
VALUES ($1, $2)
RETURNING $table_fields;