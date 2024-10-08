SELECT $table_fields
FROM users
ORDER BY id ASC
LIMIT $2 OFFSET $1;