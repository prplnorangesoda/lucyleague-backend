# LucyLeague

Here's the backend for (https://api.league.passtime.tf). Please LET ME KNOW if there's anything stupid here like exposed API keys or insecure thingies.

## Running locally

```
cargo run --features debug
```
with a `.env file` in the same directory as cargo.toml in similar format to the `.env.example`.

If you'd like to run a productionized version of the build, omit --features debug and create a .env.production file.

This will run an API server on 0.0.0.0:8080 by default!


This project is also dockerized. Simply running `docker compose up --build` after cloning (AND MAKING A `.env` AND `.env.production` FILE!) should work out of the box. Submit an issue if this doesn't work!

## Database setup

### Docker setup

`$ docker compose up --build`

Remember to set your PG\_\_HOST in .env to `db`.

Create a .env file (with .env.example). Note that for now PG DBNAME and PG USER have to be the same.


### Non docker

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
    - Note: the below code is outdated. I will update these post requests when our API is a little more well-defined.

```bash
curl -i -d '{"name":"TEST LEAGUE"}' -H 'Content-Type: application/json' http://127.0.0.1:8080/api/v1/leagues
```

```bash
echo '{"name":"TEST LEAGUE"}' | http -f --json --print h POST http://127.0.0.1:8080/api/v1/leagues
```
