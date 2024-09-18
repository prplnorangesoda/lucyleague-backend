//! The user facing API.
//! Authorization required endpoints are at the module [admin].

use crate::authorization::get_authorization_for_user;
use crate::db;
use crate::errors::MyError;
use crate::models::League;
use crate::models::MiniTeam;
use crate::models::MiniUser;
use crate::models::User;
use crate::steamapi;
use crate::CurrentHost;
use crate::PlayerSummaryAccess;
use actix_web::{get, post, web, Error, HttpResponse, Responder};
use deadpool_postgres::{Client, Pool};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

pub mod admin;
pub mod leagues;
pub mod login;
pub mod users;

mod apimodels;

use apimodels::*;

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
pub struct AppState {
    pub current_host: CurrentHost,
    pub pool: Pool,
    pub steam_auth_url: String,
    pub steam_api_key: String,
    pub root_user_steamid: Option<String>,
}

/// All parameters that a valid openid request should have.

//https://rgl.gg/Login/Default.aspx?push=1&r=40
//&dnoa.userSuppliedIdentifier=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2F
//&openid.ns=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0
//&openid.mode=id_res
//&openid.op_endpoint=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Flogin
//&openid.claimed_id=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198025040446
//&openid.identity=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198025040446
//&openid.return_to=https%3A%2F%2Frgl.gg%2FLogin%2FDefault.aspx%3Fpush%3D1%26r%3D40%26dnoa.userSuppliedIdentifier%3Dhttps%253A%252F%252Fsteamcommunity.com%252Fopenid%252F
//&openid.response_nonce=2024-07-27T16%3A07%3A06Zdg9%2BzW7ALLLycjtF7T7mWe3qKp0%3D
//&openid.assoc_handle=34321234
//&openid.signed=signed%2Cop_endpoint%2Cclaimed_id%2Cidentity%2Creturn_to%2Cresponse_nonce%2Cassoc_handle
//&openid.sig=f9dFKCcwpaGUWp2VsXwMV7csgsU%3D */
const OPENID_NECESSARY_PARAMETERS: &[&str] = &[
    "openid.ns",
    "openid.mode",
    "openid.op_endpoint",
    "openid.claimed_id",
    "openid.identity",
    "openid.return_to",
    "openid.response_nonce",
    "openid.assoc_handle",
    "openid.signed",
    "openid.sig",
];

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct OpenIdFields {
    pub openid__ns: String,
    pub openid__mode: String,
    pub openid__claimed_id: String,
    pub openid__signed: String,
    pub openid__response_nonce: String,
    pub openid__op_endpoint: String,
    pub openid__identity: String,
    pub openid__return_to: String,
    pub openid__assoc_handle: String,
    pub openid__sig: String,
}

#[derive(Serialize, Deserialize)]
struct IsOpenIdValid {
    pub valid: bool,
    pub token: Option<String>,
}
#[post("/api/v1/verifylogin")]
pub async fn verify_openid_login(
    body: web::Json<OpenIdFields>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    log::info!("POST at /api/v1/loginverify");
    let encode = serde_json::to_string(&body.0).unwrap();
    match steamapi::verify_auth_underscores(&encode).await {
        Ok(is_valid) => Ok(HttpResponse::Ok().json(IsOpenIdValid {
            valid: is_valid,
            token: None,
        })),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

#[get("/api/v1/leagues")]
async fn get_all_leagues(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    log::info!("GET request at /api/v1/leagues");
    let client = state.pool.get().await.map_err(MyError::PoolError)?;

    let leagues: Vec<League> = db::get_leagues(&client).await?;

    Ok(HttpResponse::Ok().json(leagues))
}

#[derive(Serialize, Deserialize)]
struct TeamResponse {
    pub id: i64,
    pub leagueid: i64,
    pub team_name: String,
    pub players: Vec<User>,
}

#[get("/api/v1/teams/{team_id}")]
async fn get_team(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse, Error> {
    log::info!("GET request at /api/v1/teams/{path}");
    let team_id = path.into_inner();
    if team_id < 0 {
        return Err(MyError::NotFound.into());
    }
    let client = state.pool.get().await.map_err(MyError::PoolError)?;
    let team = db::get_team_from_id(&client, team_id).await?;
    let players = db::get_team_players(&client, &team).await?;
    let resp = TeamResponse {
        id: team.id,
        leagueid: team.leagueid,
        team_name: team.team_name,
        players,
    };
    Ok(HttpResponse::Created().json(resp))
}
#[post("/api/v1/teams")]
async fn post_team(
    state: web::Data<AppState>,
    new_team: web::Json<MiniTeam>,
) -> Result<HttpResponse, Error> {
    log::info!("POST request at /api/v1/teams");
    let team = new_team.into_inner();
    let client = state.pool.get().await.map_err(MyError::PoolError)?;
    let leagueid = team.leagueid;
    let league = match db::get_league_from_id(&client, leagueid).await {
        Ok(league) => league,
        Err(_) => return Ok(HttpResponse::NotFound().body("League not found with id ${leagueid}")),
    };
    if !league.accepting_teams {
        return Ok(HttpResponse::BadRequest().body("League not accepting new teams"));
    }

    let resp = db::add_team(&client, &team).await?;
    Ok(HttpResponse::Created().json(resp))
}
