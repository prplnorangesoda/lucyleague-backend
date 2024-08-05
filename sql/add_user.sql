INSERT INTO users(steamid, username, avatarurl)
VALUES ($1, $2, $3)
RETURNING $table_fields
