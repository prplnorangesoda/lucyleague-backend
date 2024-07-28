use crate::config::ExampleConfig;
use actix_cors::Cors;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use authorization::get_authorization_for_user;
use confik::{Configuration as _, EnvSource};
use db::get_user_from_steamid;
use deadpool_postgres::{Client, Pool};
use dotenvy::dotenv;
use models::MiniUser;
use std::{collections::HashMap, io};
use tokio_postgres::NoTls;

mod authorization;
mod config;
mod db;
mod errors;
mod models;
mod openid;

use self::errors::MyError;

pub async fn get_users(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    println!("GET request at /users");
    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let users = db::get_users(&client).await?;

    Ok(HttpResponse::Ok().json(users))
}

pub async fn add_user(
    user: web::Json<MiniUser>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let user_info = user.into_inner();
    println!(
        "creating user with steamid: {0}, username: {1}",
        &user_info.steamid, &user_info.username
    );

    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let new_user = db::add_user(&client, user_info).await?;

    Ok(HttpResponse::Ok().json(new_user))
}

#[get("/api/teams/{team_id}")]
async fn get_team(path: web::Path<u32>) -> impl Responder {
    println!("GET request at /teams/id");
    let team_id = path.into_inner();
    println!("Getting info for team id {team_id}");
    if team_id != 3 {
        return HttpResponse::NotFound().body("Team id not found");
    }
    HttpResponse::Ok().body(format!("Team {team_id}"))
}

#[get("/login")]
async fn get_openid(data: web::Data<AppState>) -> impl Responder {
    println!("GET request at /login");
    HttpResponse::Found()
        .insert_header(("Location", data.into_inner().steam_auth_url.clone()))
        .body("Redirecting...")
}

/*
https://rgl.gg/Login/Default.aspx?push=1&r=40
&dnoa.userSuppliedIdentifier=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2F
&openid.ns=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0
&openid.mode=id_res
&openid.op_endpoint=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Flogin
&openid.claimed_id=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198025040446
&openid.identity=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198025040446
&openid.return_to=https%3A%2F%2Frgl.gg%2FLogin%2FDefault.aspx%3Fpush%3D1%26r%3D40%26dnoa.userSuppliedIdentifier%3Dhttps%253A%252F%252Fsteamcommunity.com%252Fopenid%252F
&openid.response_nonce=2024-07-27T16%3A07%3A06Zdg9%2BzW7ALLLycjtF7T7mWe3qKp0%3D
&openid.assoc_handle=34321234
&openid.signed=signed%2Cop_endpoint%2Cclaimed_id%2Cidentity%2Creturn_to%2Cresponse_nonce%2Cassoc_handle
&openid.sig=f9dFKCcwpaGUWp2VsXwMV7csgsU%3D */

#[get("/login/landing")]
async fn openid_landing(
    query: web::Query<HashMap<String, String>>,
    state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    println!("GET request at /login/landing");
    let inner = query.into_inner();
    // let mut keyValuesString = String::new();
    // for (key, val) in inner.iter() {
    //     keyValuesString.push_str(&format!("{key}:{val}\n"));
    // }
    // println!("{keyValuesString}");
    // let result: String = reqwest::Client::new()
    //     .post("https://steamcommunity.com/openid")
    //     .body("openid.mode=check_authentication\n")
    //     .send()
    //     .await
    //     .expect("should be a response from steam")
    //     .text()
    //     .await
    //     .expect("should be able to get text from response");
    // let openid_signed = inner
    //     .get("openid.signed")
    //     .expect("No openid.signed on request");

    let openid_identity = inner
        .get("openid.identity")
        .expect("No openid.identity on request");

    // let openid_sig = inner.get("openid.sig").expect("No openid.sig on request");
    let steamid = openid_identity.replace("https://steamcommunity.com/openid/id/", "");
    println!("Openid landing received from steamid: {steamid}");
    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let auth = match get_user_from_steamid(&client, &steamid).await {
        Ok(user) => match get_authorization_for_user(&client, &user).await {
            Ok(auth) => auth,
            Err(_) => {
                return Ok(HttpResponse::InternalServerError().body("500 Internal Server Error"))
            }
        },
        Err(err) => {
            if let MyError::NotFound = err {
                return Ok(HttpResponse::NotFound().body("404 Not Found"));
            }
            println!("user not found with steamid: {steamid}; error: {err}");
            return Err(err.into());
        }
    };
    Ok(HttpResponse::Ok()
        .append_header((
            "Set-Cookie",
            format!(
                "auth-token={0}; Expires={1}; SameSite=Lax; Path=/",
                auth.token, auth.expires
            ),
        ))
        .body(format!("{auth:?}")))
}

pub struct AppState {
    pool: Pool,
    steam_auth_url: String,
}
#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().expect("Error loading .env file");

    let config = ExampleConfig::builder()
        .override_with(EnvSource::new())
        .try_build()
        .expect("Error building config");

    let steam_config = openid::SteamOpenIdConfig::new(&format!(
        "http://{0}:{1}/login/landing",
        config.openid_realm, config.server_port
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
            }))
            .service(get_team)
            .service(
                web::resource("/api/users")
                    .route(web::get().to(get_users))
                    .route(web::post().to(add_user)),
            )
            .service(get_openid)
            .service(openid_landing)
            .service(
                actix_files::Files::new("/", "./static")
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
