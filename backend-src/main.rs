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

// Main execution and routes.
use crate::config::ExampleConfig;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use clap::Parser;
use confik::{Configuration as _, EnvSource};
use dotenvy::dotenv;
use inquire::InquireError;
use tokio::io;
use tokio_postgres::NoTls;

mod apiv1;
mod authorization;
mod config;
mod db;
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
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    println!("Making LOGGER");
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .env()
        .init()
        .unwrap();
    let args = CommandLineArgs::parse();
    log::trace!("Loading .env");
    if !cfg!(feature = "nodotenv") {
        if cfg!(feature = "debug") {
            dotenv().expect("Error loading .env file");
        } else {
            dotenvy::from_filename(".env.production").expect("Error loading .env.production file");
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

    let stmt_text = "SELECT EXISTS (SELECT FROM pg_tables WHERE tablename = 'users');";

    log::trace!("Testing if users table exists.");
    let client = pool.get().await.unwrap();
    let stmt = client.prepare(stmt_text).await.unwrap();
    let rows = client.query(&stmt, &[]).await.unwrap();
    let value: bool = rows[0].get(0);
    log::debug!("Return from Postgres: {value}");

    if !value {
        let ans = match inquire::Confirm::new(
            "No user table found. Would you like to initialize the database?",
        )
        .with_default(true)
        .prompt()
        {
            Ok(res) => res,
            Err(err) => {
                if let InquireError::NotTTY = err {
                    log::info!("This is not a TTY. Initializing database by default.");
                    true
                } else {
                    panic!("InquireError: {:?}", err);
                }
            }
        };

        if ans {
            db::initdb(&client).await.unwrap();
        }
    }

    log::debug!("Checking if users table has any entries");
    let test_users = "SELECT EXISTS (SELECT * FROM users);";
    log::trace!("Preparing query SELECT EXISTS FROM USERS");
    let test_users = client.prepare(test_users).await.unwrap();
    log::trace!("Querying");
    let rows = client.query(&test_users, &[]).await.unwrap();
    let value: bool = rows[0].get(0);

    if !value {
        let ans = match inquire::Confirm::new(
            "Users table is empty. Would you like to initialize it with some test data?",
        )
        .with_default(false)
        .prompt()
        {
            Ok(res) => res,
            Err(err) => {
                if let InquireError::NotTTY = err {
                    log::info!("This is not a TTY. Not adding test data. \n\
                    (If you are in docker, run this service via `docker compose run server` to initialize this server in a TTY.)");
                    false
                } else {
                    panic!("InquireError: {:?}", err);
                }
            }
        };

        if ans {
            db::add_test_data(&client).await.unwrap();
        }
    }

    log::info!("Using this config to run the server: {config:#?}");
    log::info!("Cors function: {0}", args.cors);
    let server_address = config.server_addr.clone();
    let cors: fn() -> Cors = match args.cors.as_str() {
        "permissive" => Cors::permissive,
        "default" => Cors::permissive,
        _ => panic!("invalid argument provided to --cors"),
    };
    let server = HttpServer::new(move || {
        let cors = cors();
        log::trace!("Inside the HttpServer closure");
        App::new()
            // NOTE: this CORS is temporary until we release to production
            // don't forget!! TODO
            .wrap(cors)
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
            .service(get_team)
            .service(users::get_user_from_steamid)
            .service(users::get_user_from_auth_token)
            .service(web::resource("/api/v1/users").route(web::get().to(users::get_users)))
            .service(admin::add_user)
            .service(leagues::get_league)
            .service(get_all_leagues)
            .service(admin::post_league)
            .service(verify_openid_login)
    })
    .bind((config.server_addr.clone(), config.server_port))?
    .workers(4)
    .run();
    log::info!(
        "API server running at http://{}:{}/",
        &config.server_addr,
        &config.server_port
    );

    server.await
}
