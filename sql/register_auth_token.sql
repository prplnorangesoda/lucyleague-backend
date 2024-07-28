INSERT INTO testing.authorization(userid, token, expires)
VALUES ($1, $2, $3)
RETURNING $fields;