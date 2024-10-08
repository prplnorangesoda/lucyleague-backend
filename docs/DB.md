## Docker setup

`$ docker compose up`

Remember to set your PG\_\_HOST in .env to `db`.

Create a .env file (with .env.example). Note that for now PG DBNAME and PG USER have to be the same.


## Non docker

To initiate the database:

1. createuser -P user  
   SQL: `CREATE USER user WITH PASSWORD 'password';`  
    Create a user (optional)

2. createdb -O user db

   SQL: `CREATE DATABASE db OWNER user;`  
    Create a database

3. Initialize the database
   `psql -f sql/initdb.sql db`

4. Grant privileges to user

5. Create a .env file (with .env.example). Note that for now PG DBNAME and PG USER have to be the same

6. Run the server.

7. Test that the server accepts POST requests.

```bash
curl -i -d '{"name":"TEST LEAGUE"}' -H 'Content-Type: application/json' http://127.0.0.1:8080/api/v1/leagues
```

```bash
echo '{"name":"TEST LEAGUE"}' | http -f --json --print h POST http://127.0.0.1:8080/api/v1/leagues
```
