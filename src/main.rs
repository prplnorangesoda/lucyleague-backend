/*
Copyright (C) 2024 Lucy Faria and collaborators (https://lucyfaria.net)

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

#![allow(dead_code)]
#![allow(unused_imports)]

use std::thread;

// Main execution and routes.
use crate::config::ExampleConfig;
use actix_cors::Cors;
use actix_extensible_rate_limit::{
    backend::{memory::InMemoryBackend, SimpleInputFunctionBuilder},
    RateLimiter,
};
use actix_web::{web, App, HttpServer};
use clap::Parser;
use confik::{Configuration as _, EnvSource};
use dotenvy::dotenv;
use tokio::io;
use tokio::time::Duration;
use tokio_postgres::NoTls;

mod apiv1;
mod authorization;
mod config;
mod db;
mod db_setup;
mod errors;
mod models;
mod openid;
mod permission;
mod steamapi;

use self::apiv1::*;
use self::steamapi::PlayerSummaryAccess;

#[derive(Debug)]
struct CurrentHost {
    address: String,
    port: u16,
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct CommandLineArgs {
    /// The level of CORS protection to add.
    ///
    /// Allowed values: `permissive`, `default`
    /// Note: currently there is no difference.
    #[arg(short, long, default_value_t = String::from("default"))]
    cors: String,

    /// Should we add test data to db?
    #[arg(long, default_value_t = false)]
    test_data: bool,
    /// Allow destructive actions to be performed to the database `ll` schema
    /// on connection (i.e. wiping a schema that doesn't have a version
    /// specified).
    ///
    /// This will only be in effect if the server is run outside
    /// of a TTY, and inquire cannot ask at runtime.
    ///
    #[arg(short = 'D', long, default_value_t = false)]
    allow_schema_destruction: bool,
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let args = CommandLineArgs::parse();
    if !cfg!(debug_assertions) {
        panic!("Unstable");
    }
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .env()
        .init()
        .unwrap();

    log::warn!("Running in unstable mode");
    thread::sleep(Duration::from_secs(1));

    log::debug!("example");
    let debug: bool = cfg!(feature = "debug") || cfg!(debug_assertions);
    log::trace!("Loading .env");
    if !cfg!(feature = "nodotenv") {
        if debug {
            match dotenvy::from_filename(".env.development") {
                Ok(dotenv) => log::info!("Loading .env from {dotenv:?}"),
                Err(error) => {
                    log::info!("Error loading .env.development, falling back to .env");
                    log::debug!("Load error: {error:?}");
                    match dotenv() {
                        Ok(dotenv) => log::info!("Loading .env from {dotenv:?}"),
                        Err(error) => panic!("Error expanding .env: {error:?}"),
                    }
                }
            }
        } else {
            match dotenvy::from_filename(".env.production") {
                Ok(dotenv) => log::info!("Loading .env from {dotenv:?}"),
                Err(error) => {
                    log::info!("Error loading .env.production, falling back to .env");
                    log::debug!("Load error: {error:?}");
                    match dotenvy::from_filename(".env") {
                        Ok(dotenv) => log::info!("Loading .env from {dotenv:?}"),
                        Err(error) => panic!("Error expanding .env: {error:?}"),
                    }
                }
            };
        }
    }

    log::trace!("Creating example config");
    let config = ExampleConfig::builder()
        .override_with(EnvSource::new())
        .try_build()
        .expect("Error building config");

    log::trace!("Creating SteamOpenIdConfig");
    let steam_config = openid::SteamOpenIdConfig::new(&format!(
        "http://{0}:{1}/api/v1/login/landing",
        &config.openid_realm, &config.openid_port
    ));

    log::trace!("Creating SteamOpenId");
    let steam_setup = openid::SteamOpenId::new(steam_config, config.clone());
    let auth_url = steam_setup.get_auth_url();

    log::trace!("Creating a database pool using deadpool_postgres");
    let pool = config.pg.create_pool(None, NoTls).unwrap();
    let client = pool.get().await.unwrap();

    let version = db_setup::version(&client).await.unwrap_or(-1);
    db_setup::migrate_from(&client, version, args.allow_schema_destruction)
        .await
        .expect("should be able to migrate (hint: if you are fine with database destruction, pass --allow-schema-destruction)");

    log::trace!("Testing if users table exists");
    let stmt_text = "SELECT EXISTS (SELECT FROM pg_tables WHERE tablename = 'users');";
    let stmt = client.prepare(stmt_text).await.unwrap();
    let rows = client.query(&stmt, &[]).await.unwrap();
    let value: bool = rows.get(0).expect("should have one row returned").get(0);
    log::debug!("Return from Postgres: {value}");

    log::debug!("Checking if users table has any entries");
    let test_users = "SELECT EXISTS (SELECT * FROM ll.users LIMIT 1);";
    log::trace!("Preparing query {test_users}");
    let test_users = client.prepare(test_users).await.unwrap();
    log::trace!("Querying");
    let rows = client.query(&test_users, &[]).await.unwrap();
    let value: bool = rows[0].get(0);

    if !value {
        if args.test_data {
            db::add_test_data(&client).await.unwrap();
        }
    }

    if debug {
        log::trace!("Using this config to run the server: {config:#?}");
    }
    log::info!("Cors function: {0}", args.cors);
    let server_address = config.server_addr.clone();
    let cors: fn() -> Cors = match args.cors.as_str() {
        "permissive" => Cors::permissive,
        "default" => Cors::permissive,
        _ => panic!("invalid argument provided to --cors: {}", &args.cors),
    };

    let workers: usize = if debug {
        4
    } else {
        std::thread::available_parallelism()?.into()
    };
    let backend = InMemoryBackend::builder().build();
    let server = HttpServer::new(move || {
        let cors = cors();
        // assign a rate limit, 30 per
        let input = SimpleInputFunctionBuilder::new(Duration::from_secs(60), 30)
            .real_ip_key()
            .build();
        let middleware = RateLimiter::builder(backend.clone(), input)
            .add_headers()
            .build();
        log::trace!("Inside the HttpServer closure");
        App::new()
            .wrap(cors)
            .wrap(middleware)
            .app_data(web::Data::new(AppState {
                current_host: CurrentHost {
                    address: server_address.clone(),
                    port: config.server_port,
                },
                pool: pool.clone(),
                steam_auth_url: auth_url.clone(),
                steam_api_key: config.steam_api_key.clone(),
                root_user_steamid: config.root_user_steamid.clone(),
            }))
            .service(teams::get_team)
            .service(teams::get_tda)
            .service(teams::post_team)
            .service(add_teams::post_team_to_league)
            .service(users::get_user_from_steamid)
            .service(users::get_user_from_auth_token)
            .service(users::get_users_paged)
            .service(users::search_users)
            // .service(admin::add_user) // this is unauthenticated...
            .service(leagues::get_league)
            .service(leagues::get_all_leagues)
            .service(admin::post_league)
            .service(admin::post_league_divisions)
            .service(verify_openid_login)
            .service(logout)
    })
    .keep_alive(Duration::from_secs(0))
    .bind((config.server_addr.clone(), config.server_port))?
    .workers(workers)
    .run();
    log::info!(
        "API server running at http://{}:{}/",
        &config.server_addr,
        &config.server_port
    );

    server.await
}
