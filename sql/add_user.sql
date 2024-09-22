INSERT INTO users(steamid, username, avatarurl, created_at)
VALUES ($1, $2, $3, $4)
RETURNING $table_fields
