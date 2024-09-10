use actix_web::http::header;
use actix_web::{http, get, post, web, Error, HttpResponse, Responder};
use actix_http::header::Header;

use crate::models::*;
use crate::apiv1::apimodels::*;
use crate::AppState;
use crate::errors::MyError;
use crate::db;
use deadpool_postgres::Client;

struct AuthHeader (String);

impl Header for AuthHeader {
    fn name() -> header::HeaderName {
        header::AUTHORIZATION
    }
    fn parse<M: actix_web::HttpMessage>(msg: &M) -> Result<Self, actix_web::error::ParseError> {
        todo!()
    }
}
#[post("/api/v1/leagues")]
pub async fn post_league(
    league: web::Json<MiniLeague>,
    state: web::Data<AppState>,
    authorization: web::Header<AuthHeader>,
) -> Result<HttpResponse, Error> {
    println!("POST request at /api/v1/leagues");
    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;
    let league = league.into_inner();
    let response = db::add_league(&client, league).await?;

    Ok(HttpResponse::Ok().json(response))
}

/// Set a user or multiple users to a team.
#[post("/api/v1/users/setteam")]
pub async fn post_users_team(
    user_team: web::Json<UserTeamBody>,
    state: web::Data<AppState>
) -> Result<HttpResponse, Error> {
    todo!();
    let client = state.pool.get().await.map_err(MyError::PoolError)?;
    let user_team = user_team.into_inner();
    let users: Vec<User> = Vec::new();
    // fetch the team to get its id
    let team = db::get_team(user_team.team_id);
    for steamid in user_team.user_steamids.iter() {
        let user = match db::get_user_from_steamid(&client, steamid).await {
            Ok(res) => res,
            Err(err) => {return Ok(HttpResponse::BadRequest().body(
                format!("Could not map {steamid} to a valid user.")
            ))}
        };
        db::add_user_team(user, team);
    }
    // fetch the new updated team to provide feedback on what the team looks like now
    let response = db::get_team_players(user_team.team_id);
    Ok(HttpResponse::Ok().json(response))
}