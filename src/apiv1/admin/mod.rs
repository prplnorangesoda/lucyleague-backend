use std::str::FromStr;

use actix_http::header::TryIntoHeaderValue;
use actix_web::http::header;
use actix_web::{http, post, web, Error, HttpResponse};
use derive_more::derive::{Debug, Display};

use super::HttpResult;
use crate::apiv1::apimodels::*;
use crate::db;
use crate::errors::MyError;
use crate::models::*;
use crate::permission::UserPermission;
use crate::AppState;
use deadpool_postgres::Client;

#[derive(Debug, Display)]
pub struct AuthHeader(pub String);

impl TryIntoHeaderValue for AuthHeader {
    type Error = actix_web::error::HttpError;
    fn try_into_value(self) -> Result<header::HeaderValue, Self::Error> {
        Ok(header::HeaderValue::from_bytes(self.0.as_bytes())?)
    }
}

pub enum AuthHeaderStringError {
    NotBearer,
}

impl FromStr for AuthHeader {
    type Err = AuthHeaderStringError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.starts_with("Bearer ") {
            true => Ok(AuthHeader(s.replacen("Bearer ", "", 1))),
            false => Err(AuthHeaderStringError::NotBearer),
        }
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

// #[post("/api/v1/admin/users")]
// pub async fn add_user(user: web::Json<MiniUser>, state: web::Data<AppState>) -> HttpResult {
//     log::info!("POST /api/v1/admin/users");
//     let user_info = user.into_inner();
//     log::debug!(
//         "creating user with steamid: {0}, username: {1}",
//         &user_info.steamid,
//         &user_info.username
//     );

//     let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

//     let new_user = db::add_user(&client, user_info).await?;

//     Ok(HttpResponse::Created().json(new_user))
// }

#[post("/api/v1/admin/leagues")]
pub async fn post_league(
    league: web::Json<MiniLeague>,
    state: web::Data<AppState>,
    authorization: web::Header<AuthHeader>,
) -> HttpResult {
    log::info!("POST /api/v1/leagues");
    log::debug!("Authorization header: {0}", authorization.0 .0);

    log::trace!("Grabbing pool");
    let client = state.pool.get().await.unwrap();

    let user = match db::get_user_from_auth_token(&client, &authorization.0 .0).await {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::Unauthorized().body("Error processing permissions")),
    };

    // if not admin / can't create league
    if !user.admin_or_perm(UserPermission::CreateLeague) {
        return Ok(HttpResponse::Forbidden().body("Insufficient permissions"));
    }

    // Actually create the new league
    log::info!("Authorization succeeded, creating a new league");
    let league = league.into_inner();
    log::debug!("Adding league from: {0:?}", league);
    let response = db::add_league(&client, league).await?;
    log::trace!("OK response, {response:?}");

    Ok(HttpResponse::Created().json(response))
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MiniDivision {
    pub leagueid: i64,
    pub name: String,
}

#[post("/api/v1/admin/divisions")]
pub async fn post_league_divisions(
    division: web::Json<MiniDivision>,
    state: web::Data<AppState>,
    auth: web::Header<AuthHeader>,
) -> HttpResult {
    log::info!("POST /api/v1/divisions");
    log::debug!("Authorization header: {0}", auth.0 .0);

    log::trace!("Grabbing pool");
    let client = crate::grab_pool(&state).await?;

    let user = match db::get_user_from_auth_token(&client, &auth.0 .0).await {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::Unauthorized().body("Error processing permissions")),
    };

    // if not admin / can't create div
    if !user.admin_or_perm(UserPermission::CreateLeague) {
        return Ok(HttpResponse::Forbidden().body("Insufficient permissions"));
    }

    // Actually create the new div
    log::info!("Authorization succeeded, creating a new division");
    let division = division.into_inner();
    log::debug!("Adding division: {0:?}", division);
    let response = db::divisions::add_division(&client, division).await?;
    log::trace!("OK response, {response:?}");

    Ok(HttpResponse::Created().json(response))
}
/// Set a user or multiple users to a team.
#[post("/api/v1/admin/setuserteam")]
pub async fn post_users_team(
    user_team: web::Json<UserTeamBody>,
    state: web::Data<AppState>,
) -> HttpResult {
    log::debug!("POST /api/v1/users/setteam");
    let client = state.pool.get().await.map_err(MyError::PoolError)?;
    let user_team = user_team.into_inner();
    // fetch the team to get its id
    let team = db::get_team_from_id(&client, user_team.team_id).await?;
    todo!();
    Ok(HttpResponse::Ok().finish())
}
