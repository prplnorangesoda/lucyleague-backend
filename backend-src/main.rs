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

// Main execution and routes.
use crate::config::ExampleConfig;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
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

#[actix_web::main]
async fn main() -> io::Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .env()
        .init()
        .unwrap();

    log::info!("Checking for /out directory");
    if !std::path::Path::new("./lucyleague-frontend/out")
        .try_exists()
        .expect("Could not check if frontend path exists")
    {
        log::error!(
            "Could not find lucyleague-frontend/out. Did you compile the frontend submodule?"
        );
        std::process::exit(1);
    };
    log::trace!("Successfully found /out dir");

    log::trace!("Loading .env");
    dotenv().expect("Error loading .env file");

    log::trace!("Creating example config");
    let config = ExampleConfig::builder()
        .override_with(EnvSource::new())
        .try_build()
        .expect("Error building config");

    log::trace!("Creating SteamOpenIdConfig");
    let steam_config = openid::SteamOpenIdConfig::new(&format!(
        "http://{0}:{1}/login/landing",
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

    log::info!("Using this config to run the server: {config:#?}");
    let server = HttpServer::new(move || {
        log::trace!("Inside the HttpServer closure");
        App::new()
            // NOTE: this CORS is temporary until we release to production
            // don't forget!! TODO
            .wrap(Cors::permissive())
            .app_data(web::Data::new(AppState {
                pool: pool.clone(),
                steam_auth_url: auth_url.clone(),
                steam_api_key: config.steam_api_key.clone(),
                root_user_steamid: config.root_user_steamid.clone(),
            }))
            .service(get_team)
            .service(get_user_from_steamid)
            .service(get_user_from_auth_token)
            .service(
                web::resource("/api/v1/users")
                    .route(web::get().to(get_users))
                    .route(web::post().to(add_user)),
            )
            .service(get_league)
            .service(get_all_leagues)
            .service(admin::post_league)
            .service(get_openid)
            .service(openid_landing)
            .service(
                actix_files::Files::new("/", "./lucyleague-frontend/out").index_file("index.html"),
            )
    })
    .bind((config.server_addr.clone(), config.server_port))?
    .run();
    log::info!(
        "Server running at http://{}:{}/",
        &config.server_addr,
        &config.server_port
    );

    server.await
}
