#!/usr/bin/env bash
# Exit on error / print whatever command we're running
set -ex
cd "$(dirname "$0")"

# build frontend
cd lucyleague-frontend
if command -v bun
then
    echo 'Running bun'
    bun install
    bun run build
else
    echo 'Running npm: this requires node'
    npm install
    npm run build
fi

cd ..

if command -v docker-compose
then
    docker-compose down
    docker-compose build && docker-compose up
else
    docker compose down
    docker compose build && docker compose up
fi

