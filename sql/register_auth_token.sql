INSERT INTO testing.authorization(userid, token)
VALUES ($1, $2)
RETURNING $fields;