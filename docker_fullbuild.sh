#!/usr/bin/env bash
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
docker-compose down
docker-compose build && docker-compose up
