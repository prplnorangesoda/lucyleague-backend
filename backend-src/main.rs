// Main execution and routes.
use crate::config::ExampleConfig;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use confik::{Configuration as _, EnvSource};
use dotenvy::dotenv;
use std::io;
use tokio_postgres::NoTls;

mod authorization;
mod checkpermission;
mod config;
mod db;
mod errors;
mod models;
mod openid;
mod steamapi;
mod apiv1;

use self::steamapi::PlayerSummaryAccess;
use self::apiv1::*;

#[actix_web::main]
async fn main() -> io::Result<()> {
    if !std::path::Path::new("./lucyleague-frontend/out")
        .try_exists()
        .expect("Could not check if frontend path exists")
    {
        panic!("Could not find lucyleague-frontend/out. Did you compile the frontend submodule?")
    };

    dotenv().expect("Error loading .env file");

    let config = ExampleConfig::builder()
        .override_with(EnvSource::new())
        .try_build()
        .expect("Error building config");

    let steam_config = openid::SteamOpenIdConfig::new(&format!(
        "http://{0}:{1}/login/landing",
        &config.openid_realm, &config.server_port
    ));

    let steam_setup = openid::SteamOpenId::new(steam_config, config.clone());
    let auth_url = steam_setup.get_auth_url();
    let pool = config.pg.create_pool(None, NoTls).unwrap();
    println!("{config:?}");

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(AppState {
                pool: pool.clone(),
                steam_auth_url: auth_url.clone(),
                steam_api_key: config.steam_api_key.clone(),
            }))
            .service(get_team)
            .service(get_user_from_steamid)
            .service(get_user_from_auth_token)
            .service(
                web::resource("/api/v1/users")
                    .route(web::get().to(get_users))
                    .route(web::post().to(add_user)),
            )
            .service(get_openid)
            .service(openid_landing)
            .service(
                actix_files::Files::new("/", "./lucyleague-frontend/out")
                    .show_files_listing()
                    .index_file("index.html"),
            )
    })
    .bind((config.server_addr.clone(), config.server_port))?
    .run();
    println!(
        "Server running at http://{}:{}/",
        &config.server_addr, &config.server_port
    );

    server.await
}
