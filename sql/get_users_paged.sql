SELECT $table_fields
FROM ll.users
ORDER BY id ASC
LIMIT $2 OFFSET $1;