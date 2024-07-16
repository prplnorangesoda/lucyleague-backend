use std::io;

use crate::config::ExampleConfig;
use actix_cors::Cors;
use actix_web::{get, web, App, Error, HttpResponse, HttpServer, Responder};
use confik::{Configuration as _, EnvSource};
use deadpool_postgres::{Client, Pool};
use dotenvy::dotenv;
use tokio_postgres::NoTls;

mod config;
mod db;
mod errors;
mod models;

use self::{errors::MyError, models::User};

pub async fn get_users(db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let client: Client = db_pool.get().await.map_err(MyError::PoolError)?;

    let users = db::get_users(&client).await?;

    Ok(HttpResponse::Ok().json(users))
}

pub async fn add_user(
    user: web::Json<User>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    let user_info: User = user.into_inner();

    let client: Client = db_pool.get().await.map_err(MyError::PoolError)?;

    let new_user = db::add_user(&client, user_info).await?;

    Ok(HttpResponse::Ok().json(new_user))
}

#[get("/teams/{team_id}")]
async fn get_team(path: web::Path<u32>) -> impl Responder {
    let team_id = path.into_inner();
    println!("Getting info for team id {team_id}");
    if team_id != 3 {
        return HttpResponse::NotFound().body("Team id not found");
    }
    HttpResponse::Ok().body(format!("Team {team_id}"))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().expect("Error loading .env file");

    let config = ExampleConfig::builder()
        .override_with(EnvSource::new())
        .try_build()
        .expect("Error building config");
    let pool = config.pg.create_pool(None, NoTls).unwrap();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(pool.clone()))
            .service(get_team)
            .service(
                web::resource("/users")
                    .route(web::get().to(get_users))
                    .route(web::post().to(add_user)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run();
    println!("Server running at http://{}/", config.server_addr);

    server.await
}
