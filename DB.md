## Docker setup

`$ docker compose up`

## Non docker

To initiate the database:

1. createuser -P user  
   `CREATE USER user WITH PASSWORD 'password';`  
    Create a user (optional)

2. createdb -O user db  
   `CREATE DATABASE db OWNER user;`  
    Create a database

3. Initialize the database
   `psql -f sql/initdb.sql db`

4. Grant privileges to user

5. Create .env file (with .env.example)

6. Run the server.

7. Test that the server accepts POST requests.

```bash
curl -i -d '{"steamid": "654", "username": "example"}' -H 'Content-Type: application/json' http://127.0.0.1:8080/users
```

```bash
echo '{"steamid": "654", "username": "example"}' | http -f --json --print h POST http://127.0.0.1:8080/users
```
