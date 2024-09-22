SELECT 
	$table_fields 
FROM 
	users
WHERE
	username LIKE CONCAT('%', $1::text, '%')
	OR
	steamid LIKE CONCAT('%', $1::text, '%')
ORDER BY id ASC
LIMIT $2 
OFFSET $3;