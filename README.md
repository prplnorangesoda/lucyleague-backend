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
