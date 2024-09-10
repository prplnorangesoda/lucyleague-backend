use std::str::FromStr;

use actix_http::header::TryIntoHeaderValue;
use actix_web::http::header;
use actix_web::{get, http, post, web, Error, HttpResponse, Responder};

use crate::apiv1::apimodels::*;
use crate::db;
use crate::errors::MyError;
use crate::models::*;
use crate::AppState;
use deadpool_postgres::Client;

struct AuthHeader(String);

impl TryIntoHeaderValue for AuthHeader {
    type Error = actix_web::error::HttpError;
    fn try_into_value(self) -> Result<header::HeaderValue, Self::Error> {
        Ok(header::HeaderValue::from_bytes(self.0.as_bytes())?)
    }
}

pub struct AuthHeaderStringError;

impl FromStr for AuthHeader {
    type Err = AuthHeaderStringError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AuthHeader("Hi".to_owned()))
    }
}
impl http::header::Header for AuthHeader {
    fn name() -> header::HeaderName {
        header::AUTHORIZATION
    }
    fn parse<M: actix_web::HttpMessage>(msg: &M) -> Result<Self, actix_web::error::ParseError> {
        actix_http::header::from_one_raw_str(msg.headers().get(Self::name()))
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
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let client = state.pool.get().await.map_err(MyError::PoolError)?;
    let user_team = user_team.into_inner();
    // fetch the team to get its id
    let team = db::get_team_from_id(&client, user_team.team_id).await?;
    let users = db::get_team_players(&client, team).await?;
    Ok(HttpResponse::Ok().json(users))
}
