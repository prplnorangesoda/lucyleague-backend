// Main execution and routes.
use crate::config::ExampleConfig;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use confik::{Configuration as _, EnvSource};
use dotenvy::dotenv;
use inquire::InquireError;
use tokio::io::{self, AsyncReadExt};
use tokio_postgres::NoTls;

mod apiv1;
mod authorization;
mod checkpermission;
mod config;
mod db;
mod errors;
mod models;
mod openid;
mod steamapi;

use self::apiv1::*;
use self::steamapi::PlayerSummaryAccess;

#[actix_web::main]
async fn main() -> io::Result<()> {
    if !std::path::Path::new("./lucyleague-frontend/out")
        .try_exists()
        .expect("Could not check if frontend path exists")
    {
        eprintln!("Could not find lucyleague-frontend/out. Did you compile the frontend submodule?");
        std::process::exit(1);
    };

    dotenv().expect("Error loading .env file");

    let config = ExampleConfig::builder()
        .override_with(EnvSource::new())
        .try_build()
        .expect("Error building config");

    let steam_config = openid::SteamOpenIdConfig::new(&format!(
        "http://{0}:{1}/login/landing",
        &config.openid_realm, &config.openid_port
    ));

    let steam_setup = openid::SteamOpenId::new(steam_config, config.clone());
    let auth_url = steam_setup.get_auth_url();
    let pool = config.pg.create_pool(None, NoTls).unwrap();
    
    let stmt_text = "SELECT EXISTS (SELECT FROM pg_tables WHERE tablename = 'users');";

    let client = pool.get().await.unwrap();
    let stmt = client.prepare(stmt_text).await.unwrap();
    let rows = client.query(&stmt, &[]).await.unwrap();
    let value: bool = rows[0].get(0);

    
    if !value {
        let ans = match inquire::Confirm::new("No user table found. Would you like to initialize the database?")
        .with_default(true)
        .prompt() {
            Ok(res) => res,
            Err(err) => {
                if let InquireError::NotTTY = err {
                    eprintln!("This is not a TTY. Initializing database by default.");
                    true
                }
                else {
                    panic!("InquireError: {:?}", E);
                }
            }
        };

        if ans {
            db::initdb(&client).await.unwrap();
        }
        else {
            panic!("{:?}", ans);
        }
        
    }

    println!("{client:?}");
    println!("{config:?}");

    let server = HttpServer::new(move || {
        App::new()
            // NOTE: this CORS is temporary until we release to production
            // don't forget!! TODO
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
            .service(get_league)
            .service(get_all_leagues)
            .service(post_league)
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
